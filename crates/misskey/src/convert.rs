//! The wire mapping. Misskey's shapes stay in this file.

use nodus_client_core::model::{
    Account, CredentialSource, CustomEmoji, LocalId, MediaAttachment, MediaKind, Mention,
    Notification, NotificationKind, Poll, PollOption, ProfileField, Relationship, Status, Tag,
    Visibility,
};
use serde_json::Value;

/// Map a Misskey user to the account shape the views render.
pub fn account(value: &Value) -> Option<Account> {
    let username = value.get("username")?.as_str()?;
    let host = value.get("host").and_then(Value::as_str);
    let acct = host.map_or_else(|| username.to_owned(), |host| format!("{username}@{host}"));
    let emojis: Vec<CustomEmoji> = value
        .get("emojis")
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .filter_map(|(shortcode, url)| {
                    url.as_str().map(|url| CustomEmoji {
                        shortcode: shortcode.clone(),
                        url: url.to_owned(),
                        static_url: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(Account {
        id: string(value, "id")?,
        acct,
        display_name: value
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        note: text_html(
            value
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            &emojis,
        ),
        avatar: value
            .get("avatarUrl")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        header: value
            .get("bannerUrl")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        bot: value.get("isBot").and_then(Value::as_bool).unwrap_or(false),
        locked: value
            .get("isLocked")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        followers_count: number(value, "followersCount"),
        following_count: number(value, "followingCount"),
        statuses_count: number(value, "notesCount"),
        last_status_at: None,
        discoverable: value.get("isExplorable").and_then(Value::as_bool),
        indexable: None,
        noindex: None,
        fields: profile_fields(value),
    })
}

/// Map Misskey's `fields` rows to the shared name/value shape.
///
/// Misskey has no verification proof, so every row arrives without `verified_at`.
fn profile_fields(value: &Value) -> Vec<ProfileField> {
    value
        .get("fields")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    Some(ProfileField::new(
                        row.get("name")?.as_str()?,
                        row.get("value")?.as_str()?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Map a Misskey note to the status shape the views render.
pub fn status(value: Value) -> Option<Status> {
    let account = account(value.get("user")?)?;
    let emojis = emojis(&value);
    let files: Vec<MediaAttachment> = value
        .get("files")
        .and_then(Value::as_array)
        .map(|files| files.iter().filter_map(media).collect())
        .unwrap_or_default();
    let renote = value.get("renote").cloned().and_then(status);
    let text = value
        .get("text")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let cw = value.get("cw").and_then(Value::as_str).unwrap_or_default();
    let reactions = value.get("reactions").and_then(Value::as_object);
    let favourites = reactions.map_or(0, |map| map.values().filter_map(Value::as_u64).sum::<u64>());
    Some(Status {
        id: string(&value, "id")?,
        created_at: string(&value, "createdAt").unwrap_or_default(),
        content: text_html(text, &emojis),
        content_format: Some("mfm".to_owned()),
        content_source: Some(text.to_owned()),
        visibility: visibility_in(value.get("visibility").and_then(Value::as_str)),
        account,
        favourites_count: favourites,
        reblogs_count: number(&value, "renoteCount"),
        replies_count: number(&value, "repliesCount"),
        favourited: value.get("myReaction").and_then(Value::as_str).is_some(),
        reblogged: false,
        bookmarked: boolean(&value, "isFavorited"),
        reblog: renote.map(Box::new),
        in_reply_to_id: string(&value, "replyId"),
        emojis,
        url: value.get("url").and_then(Value::as_str).map(str::to_owned),
        sensitive: files.iter().any(|media| media.sensitive),
        spoiler_text: cw.to_owned(),
        language: None,
        media_attachments: files,
        mentions: Vec::<Mention>::new(),
        tags: tags(text),
        poll: value.get("poll").cloned().and_then(poll),
        edited_at: None,
        pinned: None,
    })
}

pub fn notification(value: Value) -> Option<Notification> {
    let account = account(value.get("user")?)?;
    let kind = match value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "follow" => NotificationKind::Follow,
        "receiveFollowRequest" => NotificationKind::FollowRequest,
        "renote" | "quote" => NotificationKind::Reblog,
        "reaction" | "like" => NotificationKind::Favourite,
        "mention" | "reply" => NotificationKind::Mention,
        "pollEnded" => NotificationKind::Poll,
        "note" => NotificationKind::Status,
        _ => NotificationKind::Unknown,
    };
    Some(Notification {
        id: string(&value, "id")?,
        kind,
        created_at: string(&value, "createdAt").unwrap_or_default(),
        account,
        status: value.get("note").cloned().and_then(status),
    })
}

pub fn relationship(value: &Value, id: &LocalId) -> Option<Relationship> {
    Some(Relationship {
        id: id.clone(),
        following: boolean(value, "isFollowing"),
        requested: boolean(value, "isFollowRequested"),
        followed_by: boolean(value, "isFollowed"),
        blocking: boolean(value, "isBlocking"),
        blocked_by: boolean(value, "isBlocked"),
        muting: boolean(value, "isMuted"),
        muting_notifications: boolean(value, "isRenoteMuted"),
        domain_blocking: false,
        endorsed: false,
    })
}

pub fn source(value: &Value) -> CredentialSource {
    let visibility = value
        .pointer("/clientData/defaultNoteVisibility")
        .and_then(Value::as_str)
        .or_else(|| value.get("defaultNoteVisibility").and_then(Value::as_str));
    CredentialSource {
        privacy: visibility.map(str::to_owned),
        sensitive: None,
        language: None,
    }
}

/// Misskey sends MFM, not HTML. Escape it, keep line breaks, and paint custom emoji.
#[must_use]
pub fn note_html(text: &str, emojis: &[CustomEmoji]) -> String {
    text_html(text, emojis)
}

fn text_html(text: &str, emojis: &[CustomEmoji]) -> String {
    let mut html = String::with_capacity(text.len() + 16);
    for (line_index, line) in text.split('\n').enumerate() {
        if line_index > 0 {
            html.push_str("<br>");
        }
        let line = line.strip_suffix('\r').unwrap_or(line);
        for (token_index, token) in line.split_whitespace().enumerate() {
            if token_index > 0 {
                html.push(' ');
            }
            if let Some(shortcode) = token
                .strip_prefix(':')
                .and_then(|rest| rest.strip_suffix(':'))
            {
                if let Some(emoji) = emojis.iter().find(|emoji| emoji.shortcode == shortcode) {
                    let escaped_shortcode = escape(shortcode);
                    html.push_str(&format!(
                        "<img class=\"emoji\" alt=\":{escaped_shortcode}:\" title=\":{escaped_shortcode}:\" src=\"{}\">",
                        escape(&emoji.url)
                    ));
                    continue;
                }
            }
            html.push_str(&escape(token));
        }
    }
    html
}

fn media(value: &Value) -> Option<MediaAttachment> {
    let kind = match value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "image" => MediaKind::Image,
        "video" => MediaKind::Video,
        "audio" => MediaKind::Audio,
        "gifv" => MediaKind::Gifv,
        _ => MediaKind::Unknown,
    };
    let properties = value.get("properties");
    Some(MediaAttachment {
        id: string(value, "id")?,
        kind,
        url: value.get("url").and_then(Value::as_str).map(str::to_owned),
        preview_url: value
            .get("thumbnailUrl")
            .and_then(Value::as_str)
            .map(str::to_owned),
        description: value
            .get("comment")
            .and_then(Value::as_str)
            .map(str::to_owned),
        blurhash: None,
        meta_width: properties
            .and_then(|properties| properties.get("width"))
            .and_then(Value::as_u64)
            .map(|width| width as u32),
        meta_height: properties
            .and_then(|properties| properties.get("height"))
            .and_then(Value::as_u64)
            .map(|height| height as u32),
        sensitive: boolean(value, "isSensitive"),
        processing_state: None,
        processing_progress: None,
    })
}

fn poll(value: Value) -> Option<Poll> {
    poll_with_now(value, now_unix_millis())
}

fn poll_with_now(value: Value, now_ms: i64) -> Option<Poll> {
    let choices = value.get("choices")?.as_array()?;
    let mut options = Vec::with_capacity(choices.len());
    let mut own_votes = Vec::new();
    for choice in choices {
        let Some(title) = choice.get("text").and_then(Value::as_str) else {
            continue;
        };
        let index = options.len();
        options.push(PollOption {
            title: title.to_owned(),
            votes_count: choice.get("votes").and_then(Value::as_u64),
        });
        if boolean(choice, "isVoted") {
            own_votes.push(index);
        }
    }
    let votes_count = options
        .iter()
        .filter_map(|option| option.votes_count)
        .sum::<u64>();
    let expires_at = string(&value, "expiresAt");
    let expired = value
        .get("expired")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| {
            expires_at
                .as_deref()
                .and_then(parse_iso8601_millis)
                .is_some_and(|at_ms| at_ms <= now_ms)
        });
    Some(Poll {
        id: String::new(),
        expires_at,
        expired,
        multiple: boolean(&value, "multiple"),
        votes_count,
        voters_count: None,
        voted: !own_votes.is_empty(),
        own_votes,
        options,
        emojis: Vec::new(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn now_unix_millis() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
}

#[cfg(target_arch = "wasm32")]
fn now_unix_millis() -> i64 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "epoch milliseconds fit i64 for the next 292 million years"
    )]
    {
        js_sys::Date::now() as i64
    }
}

fn parse_iso8601_millis(raw: &str) -> Option<i64> {
    let bytes = raw.as_bytes();
    if bytes.len() < 20 {
        return None;
    }
    let year = parse_digits(bytes.get(0..4)?)?;
    if bytes[4] != b'-' {
        return None;
    }
    let month = parse_digits(bytes.get(5..7)?)?;
    if bytes[7] != b'-' {
        return None;
    }
    let day = parse_digits(bytes.get(8..10)?)?;
    if !matches!(bytes[10], b'T' | b't' | b' ') {
        return None;
    }
    let hour = parse_digits(bytes.get(11..13)?)?;
    if bytes[13] != b':' {
        return None;
    }
    let minute = parse_digits(bytes.get(14..16)?)?;
    if bytes[16] != b':' {
        return None;
    }
    let second = parse_digits(bytes.get(17..19)?)?;

    if !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }

    let mut pos = 19;
    let mut frac_ms = 0_i64;
    if bytes.get(pos) == Some(&b'.') {
        pos += 1;
        let frac_start = pos;
        let mut scale = 100_i64;
        while let Some(&b) = bytes.get(pos) {
            if !b.is_ascii_digit() {
                break;
            }
            if scale > 0 {
                frac_ms += i64::from(b - b'0') * scale;
                scale /= 10;
            }
            pos += 1;
        }
        if pos == frac_start {
            return None;
        }
    }

    let tz_offset_secs = match bytes.get(pos)? {
        b'Z' | b'z' => {
            pos += 1;
            0_i64
        }
        sign @ (b'+' | b'-') => {
            pos += 1;
            let tz_hour = parse_digits(bytes.get(pos..pos + 2)?)?;
            pos += 2;
            if bytes.get(pos) == Some(&b':') {
                pos += 1;
            }
            let tz_min = parse_digits(bytes.get(pos..pos + 2)?)?;
            pos += 2;
            if tz_hour > 23 || tz_min > 59 {
                return None;
            }
            let offset = tz_hour * 3600 + tz_min * 60;
            if *sign == b'-' { -offset } else { offset }
        }
        _ => return None,
    };

    if pos != bytes.len() {
        return None;
    }

    let days = days_from_civil(year, month, day);
    let secs = days * 86_400 + hour * 3600 + minute * 60 + second - tz_offset_secs;
    Some(secs * 1000 + frac_ms)
}

fn parse_digits(bytes: &[u8]) -> Option<i64> {
    let mut value = 0_i64;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        value = value * 10 + i64::from(b - b'0');
    }
    Some(value)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 0,
    }
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = year - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn visibility_out(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Public => "public",
        Visibility::Unlisted => "home",
        Visibility::Private => "followers",
        Visibility::Direct => "specified",
    }
}

fn visibility_in(value: Option<&str>) -> Visibility {
    match value.unwrap_or("public") {
        "home" => Visibility::Unlisted,
        "followers" => Visibility::Private,
        "specified" => Visibility::Direct,
        _ => Visibility::Public,
    }
}

fn emojis(value: &Value) -> Vec<CustomEmoji> {
    value
        .get("emojis")
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .filter_map(|(shortcode, url)| {
                    url.as_str().map(|url| CustomEmoji {
                        shortcode: shortcode.clone(),
                        url: url.to_owned(),
                        static_url: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn tags(text: &str) -> Vec<Tag> {
    text.split_whitespace()
        .filter(|word| word.starts_with('#') && word.len() > 1)
        .map(|word| Tag {
            id: None,
            name: word.trim_start_matches('#').to_owned(),
            url: String::new(),
            history: Vec::new(),
            following: false,
            statuses_count: None,
            following_count: None,
        })
        .collect()
}

fn string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn number(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn boolean(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_misskey_note_becomes_a_status_with_media_and_emoji() {
        let note = serde_json::json!({
            "id": "9abc",
            "createdAt": "2026-11-12T03:00:00.000Z",
            "text": "hello :blob: world",
            "cw": null,
            "visibility": "home",
            "user": {
                "id": "u1",
                "username": "alice",
                "name": "Alice",
                "host": "misskey.example",
                "avatarUrl": "https://misskey.example/a.png",
                "followersCount": 3,
                "followingCount": 4,
                "notesCount": 5
            },
            "files": [{
                "id": "f1",
                "type": "image",
                "url": "https://misskey.example/f.png",
                "thumbnailUrl": "https://misskey.example/t.png",
                "comment": "alt",
                "properties": { "width": 800, "height": 600 }
            }],
            "emojis": { "blob": "https://misskey.example/blob.png" },
            "reactions": { "like": 2, ":blob:": 1 },
            "renoteCount": 1,
            "repliesCount": 2,
            "url": "https://misskey.example/notes/9abc"
        });
        let status = status(note).expect("status");
        assert_eq!(status.account.acct, "alice@misskey.example");
        assert_eq!(status.visibility, Visibility::Unlisted);
        assert_eq!(status.favourites_count, 3);
        assert_eq!(status.media_attachments[0].meta_width, Some(800));
        assert!(status.content.contains("<img class=\"emoji\""));
        assert!(status.content.contains(":blob:"));
    }

    #[test]
    fn a_local_misskey_user_has_a_bare_acct() {
        let user = serde_json::json!({
            "id": "u1",
            "username": "alice",
            "host": null
        });
        assert_eq!(account(&user).expect("account").acct, "alice");
    }

    #[test]
    fn misskey_profile_rows_keep_their_name_and_value() {
        // Misskey has no verification proof, so the rows arrive unverified; dropping
        // them would leave the model's `fields` empty for every Misskey account.
        let user = serde_json::json!({
            "id": "u1",
            "username": "alice",
            "fields": [
                {"name": "Website", "value": "https://alice.test"},
                {"name": "Pronouns", "value": "she/her"}
            ]
        });

        let account = account(&user).expect("account");

        assert_eq!(account.fields.len(), 2);
        assert_eq!(account.fields[0].name, "Website");
        assert_eq!(account.fields[0].verified_at, None);
    }

    #[test]
    fn misskey_visibility_names_map_back_to_mastodon_names() {
        assert_eq!(visibility_out(Visibility::Public), "public");
        assert_eq!(visibility_out(Visibility::Unlisted), "home");
        assert_eq!(visibility_out(Visibility::Private), "followers");
        assert_eq!(visibility_out(Visibility::Direct), "specified");
    }

    #[test]
    fn emoji_shortcode_is_html_escaped() {
        let emojis = vec![
            CustomEmoji {
                shortcode: r#"blob"onload="alert(1)"#.to_owned(),
                url: "https://misskey.example/blob.png".to_owned(),
                static_url: None,
            },
            CustomEmoji {
                shortcode: "<script>".to_owned(),
                url: "https://misskey.example/s.png".to_owned(),
                static_url: None,
            },
        ];
        let html = note_html(r#":blob"onload="alert(1): :<script>:"#, &emojis);
        assert!(
            !html.contains(r#"onload="alert(1)""#),
            "attribute breakout must be escaped, got: {html}"
        );
        assert!(
            html.contains(r#"alt=":blob&quot;onload=&quot;alert(1):""#),
            "alt must escape quotes, got: {html}"
        );
        assert!(
            html.contains(r#"title=":blob&quot;onload=&quot;alert(1):""#),
            "title must escape quotes, got: {html}"
        );
        assert!(
            !html.contains("<script>"),
            "angle brackets in shortcode must be escaped, got: {html}"
        );
        assert!(
            html.contains(r#"alt=":&lt;script&gt;:""#),
            "alt must escape angle brackets, got: {html}"
        );
    }

    #[test]
    fn note_html_preserves_line_breaks_and_renders_emojis() {
        let emojis = vec![CustomEmoji {
            shortcode: "blob".to_owned(),
            url: "https://misskey.example/blob.png".to_owned(),
            static_url: None,
        }];
        let html = note_html("first line\nsecond :blob: line\n\nthird line", &emojis);
        assert_eq!(
            html,
            "first line<br>second <img class=\"emoji\" alt=\":blob:\" title=\":blob:\" src=\"https://misskey.example/blob.png\"> line<br><br>third line"
        );
        assert_eq!(note_html("a\r\nb", &[]), "a<br>b");
        assert_eq!(note_html("\nhello", &[]), "<br>hello");
    }

    #[test]
    fn alt_text_does_not_mark_status_sensitive_and_reads_is_sensitive() {
        let base_user = serde_json::json!({
            "id": "u1",
            "username": "alice"
        });
        let alt_only_note = serde_json::json!({
            "id": "n1",
            "createdAt": "2026-11-12T03:00:00.000Z",
            "text": "photo with alt",
            "user": base_user,
            "files": [{
                "id": "f1",
                "type": "image",
                "url": "https://misskey.example/f1.png",
                "comment": "A cat sitting on a keyboard",
                "isSensitive": false
            }]
        });
        let alt_status = status(alt_only_note).expect("status");
        assert_eq!(
            alt_status.media_attachments[0].description.as_deref(),
            Some("A cat sitting on a keyboard")
        );
        assert!(!alt_status.media_attachments[0].sensitive);
        assert!(!alt_status.sensitive);

        let nsfw_note = serde_json::json!({
            "id": "n2",
            "createdAt": "2026-11-12T03:00:00.000Z",
            "text": "sensitive photo",
            "user": base_user,
            "files": [{
                "id": "f2",
                "type": "image",
                "url": "https://misskey.example/f2.png",
                "isSensitive": true
            }]
        });
        let nsfw_status = status(nsfw_note).expect("status");
        assert!(nsfw_status.media_attachments[0].sensitive);
        assert!(nsfw_status.sensitive);
    }

    #[test]
    fn status_maps_my_reaction_and_is_favorited() {
        let note = serde_json::json!({
            "id": "n1",
            "createdAt": "2026-11-12T03:00:00.000Z",
            "text": "hi",
            "user": { "id": "u1", "username": "alice" },
            "myReaction": "❤️",
            "isFavorited": true
        });
        let mapped = status(note).expect("status");
        assert!(mapped.favourited);
        assert!(mapped.bookmarked);
        assert!(!mapped.reblogged);
    }

    #[test]
    fn poll_checks_expiry_and_maps_own_votes() {
        let expired_note = serde_json::json!({
            "id": "n1",
            "createdAt": "2020-01-01T00:00:00.000Z",
            "text": "old poll",
            "user": { "id": "u1", "username": "alice" },
            "poll": {
                "multiple": true,
                "expiresAt": "2020-01-02T00:00:00.000Z",
                "choices": [
                    { "text": "A", "votes": 3, "isVoted": false },
                    { "text": "B", "votes": 5, "isVoted": true }
                ]
            }
        });
        let expired_poll = status(expired_note).expect("status").poll.expect("poll");
        assert!(expired_poll.expired);
        assert!(expired_poll.multiple);
        assert!(expired_poll.voted);
        assert_eq!(expired_poll.own_votes, vec![1]);
        assert_eq!(expired_poll.votes_count, 8);

        let open_note = serde_json::json!({
            "id": "n2",
            "createdAt": "2026-11-12T00:00:00.000Z",
            "text": "open poll",
            "user": { "id": "u1", "username": "alice" },
            "poll": {
                "multiple": false,
                "expiresAt": "2099-01-01T00:00:00.000Z",
                "choices": [
                    { "text": "Yes", "votes": 1, "isVoted": false },
                    { "text": "No", "votes": 0, "isVoted": false }
                ]
            }
        });
        let open_poll = status(open_note).expect("status").poll.expect("poll");
        assert!(!open_poll.expired);
        assert!(!open_poll.voted);
        assert_eq!(open_poll.own_votes, Vec::<usize>::new());
    }

    #[test]
    fn poll_with_now_handles_timezones_boundaries_and_null_expiry() {
        let base_ms = parse_iso8601_millis("2026-11-15T12:00:00.000Z").expect("valid timestamp");
        assert_eq!(
            parse_iso8601_millis("2026-11-15T21:00:00+09:00"),
            Some(base_ms)
        );
        assert_eq!(
            parse_iso8601_millis("2026-11-15T12:00:00.5Z"),
            Some(base_ms + 500)
        );
        assert_eq!(parse_iso8601_millis("2026-02-29T00:00:00Z"), None);
        assert!(parse_iso8601_millis("2024-02-29T00:00:00Z").is_some());

        let poll_json = serde_json::json!({
            "expiresAt": "2026-11-15T21:00:00+09:00",
            "choices": [{ "text": "A", "votes": 1 }]
        });
        assert!(
            !poll_with_now(poll_json.clone(), base_ms - 1)
                .expect("poll")
                .expired
        );
        assert!(poll_with_now(poll_json, base_ms).expect("poll").expired);

        let timeless = serde_json::json!({
            "expiresAt": null,
            "choices": [{ "text": "A", "votes": 0 }]
        });
        assert!(!poll_with_now(timeless, base_ms).expect("poll").expired);
    }
}
