//! The rest of a status, and the shapes a "full client" needs beside it.
//!
//! Split from `model/mod.rs` because a status is one type and these are the things
//! that hang off it: media, mentions, tags, polls, edit history. Keeping them apart
//! keeps each file about one question, and lets this file grow as the client grows
//! without the status definition drifting out of view.

use serde::{Deserialize, Serialize};

use super::{Account, CustomEmoji, Status};

/// An account named in the content of a status.
///
/// Separate from [`Account`] on purpose: a mention carries no counts and no
/// avatar, only what a renderer needs to turn `@acct` into a link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mention {
    /// Instance-local id.
    pub id: String,
    /// Local username, without the domain.
    pub username: String,
    /// `user@host`, always fully qualified.
    pub acct: String,
    /// Profile URL.
    #[serde(default)]
    pub url: String,
}

/// A hashtag in the content of a status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    /// The tag's own id, when the payload carries one.
    #[serde(default)]
    pub id: Option<String>,
    /// The tag itself, without `#`.
    pub name: String,
    /// Permalink to the tag timeline.
    #[serde(default)]
    pub url: String,
    /// Daily usage counts, when the instance reports them.
    #[serde(default)]
    pub history: Vec<TagHistory>,
    /// Whether the signed-in account follows this tag.
    #[serde(default)]
    pub following: bool,
    /// Posts that used the tag over the instance's retention window.
    #[serde(default)]
    pub statuses_count: Option<u64>,
    /// Accounts following the tag.
    #[serde(default)]
    pub following_count: Option<u64>,
}

/// One day of usage for a hashtag.
///
/// The counts arrive as strings, which is Mastodon's quirk, not ours: parsing them
/// into numbers here would make a malformed day fail the whole tag payload instead of
/// costing one entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagHistory {
    /// `YYYY-MM-DD`.
    pub day: String,
    /// Accounts that used the tag that day, as a decimal string.
    #[serde(default)]
    pub accounts: String,
    /// Posts that used it, as a decimal string.
    #[serde(default)]
    pub uses: String,
}

/// The kind of an attachment, as the wire names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    /// Still image.
    Image,
    /// Video file.
    Video,
    /// Silent looping video (Mastodon's `gifv`).
    Gifv,
    /// Audio file.
    Audio,
    /// A type this client predates.
    ///
    /// Rendered as a link rather than dropped: an unknown attachment still exists,
    /// and hiding it would misrepresent the post.
    #[serde(other)]
    Unknown,
}

/// One attachment on a status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Media {
    /// Instance-local id.
    pub id: String,
    /// What it is.
    #[serde(rename = "type")]
    pub kind: MediaKind,
    /// The file, when the instance will serve it.
    #[serde(default)]
    pub url: Option<String>,
    /// A smaller copy, for grids.
    #[serde(default)]
    pub preview_url: Option<String>,
    /// Alt text, written by the author.
    #[serde(default)]
    pub description: Option<String>,
    /// Placeholder hash, for rendering before the bytes arrive.
    #[serde(default)]
    pub blurhash: Option<String>,
    /// Width in pixels, when reported.
    #[serde(default)]
    pub meta_width: Option<u32>,
    /// Height in pixels, when reported.
    #[serde(default)]
    pub meta_height: Option<u32>,
    /// Whether the author marked this attachment as sensitive.
    #[serde(default)]
    pub sensitive: bool,
    /// Server-side processing state: `pending`, `processing`, `ready`, `failed`.
    #[serde(default)]
    pub processing_state: Option<String>,
    /// Processing progress from 0 to 100.
    #[serde(default)]
    pub processing_progress: Option<u8>,
}

/// Mastodon's name for [`Media`].
pub use Media as MediaAttachment;

/// A poll attached to a status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Poll {
    /// Poll id.
    pub id: String,
    /// When voting closes.
    #[serde(default)]
    pub expires_at: Option<String>,
    /// Whether voting has closed.
    #[serde(default)]
    pub expired: bool,
    /// Whether several options may be picked.
    #[serde(default)]
    pub multiple: bool,
    /// Total votes.
    #[serde(default)]
    pub votes_count: u64,
    /// Distinct voters, `None` when the instance hides them.
    #[serde(default)]
    pub voters_count: Option<u64>,
    /// Whether the signed-in account voted.
    #[serde(default)]
    pub voted: bool,
    /// Which options the signed-in account picked.
    #[serde(default)]
    pub own_votes: Vec<usize>,
    /// The options.
    #[serde(default)]
    pub options: Vec<PollOption>,
    /// Emoji used in the options.
    #[serde(default)]
    pub emojis: Vec<CustomEmoji>,
}

/// One option of a poll.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PollOption {
    /// The label, plain text.
    pub title: String,
    /// Votes for it, `None` while the instance withholds counts.
    #[serde(default)]
    pub votes_count: Option<u64>,
}

/// One revision of a status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The content as it read at the time.
    #[serde(default)]
    pub content: String,
    /// The content warning as it read at the time.
    #[serde(default)]
    pub spoiler_text: String,
    /// Whether it was marked sensitive then.
    #[serde(default)]
    pub sensitive: bool,
    /// When that revision was written.
    pub created_at: String,
    /// Who edited it.
    #[serde(default)]
    pub account: Option<Account>,
}

/// A list, Mastodon's way of narrowing the home timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct List {
    /// List id.
    pub id: String,
    /// The name the owner gave it.
    pub title: String,
    /// `followed`, `list` or `none` — who may reply.
    #[serde(default)]
    pub replies_policy: String,
    /// Whether members' posts appear in the home timeline at all.
    #[serde(default)]
    pub exclusive: Option<bool>,
}

/// A direct-message thread.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conversation {
    /// Conversation id, used for reading it.
    pub id: String,
    /// Whether anything in it is unread.
    #[serde(default)]
    pub unread: bool,
    /// Everyone in the thread.
    #[serde(default)]
    pub accounts: Vec<Account>,
    /// The newest status, which is what a list row shows.
    #[serde(default)]
    pub last_status: Option<Status>,
}

/// What `/api/v2/search` answers.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchResults {
    /// Matching accounts.
    #[serde(default)]
    pub accounts: Vec<Account>,
    /// Matching statuses.
    #[serde(default)]
    pub statuses: Vec<Status>,
    /// Matching hashtags.
    #[serde(default)]
    pub hashtags: Vec<Tag>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_attachment_kind_survives_parsing() {
        let json = serde_json::json!({ "id": "1", "type": "hologram" });
        let media: MediaAttachment = serde_json::from_value(json).expect("media");
        assert_eq!(media.kind, MediaKind::Unknown);
    }

    #[test]
    fn a_status_without_any_of_the_new_fields_still_parses() {
        let json = serde_json::json!({
            "id": "1",
            "created_at": "2024-01-01T00:00:00Z",
            "visibility": "public",
            "account": {
                "id": "9", "acct": "a@b", "display_name": "A",
                "note": "", "avatar": "", "header": "",
                "bot": false, "locked": false,
                "followers_count": 0, "following_count": 0, "statuses_count": 0
            }
        });
        let status: Status = serde_json::from_value(json).expect("status");
        assert_eq!(status.media_attachments, Vec::<MediaAttachment>::new());
        assert!(status.poll.is_none());
        assert_eq!(status.spoiler_text, "");
    }

    #[test]
    fn poll_votes_round_trip_through_the_own_votes_list() {
        let poll = Poll {
            id: "1".to_owned(),
            expires_at: None,
            expired: false,
            multiple: true,
            votes_count: 3,
            voters_count: Some(2),
            voted: true,
            own_votes: vec![0, 2],
            options: vec![
                PollOption {
                    title: "a".to_owned(),
                    votes_count: Some(1),
                },
                PollOption {
                    title: "b".to_owned(),
                    votes_count: None,
                },
            ],
            emojis: Vec::new(),
        };
        let json = serde_json::to_string(&poll).expect("json");
        let back: Poll = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, poll);
    }
}

/// What an account may change about its own profile.
///
/// Every field is optional and omitted fields are left alone: the wire treats an absent
/// key as "unchanged", and a client that always sent the whole profile would overwrite a
/// bio typed in another tab.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ProfileUpdate {
    /// The display name; an empty string is a real value, not a missing one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The bio, as plain text or limited HTML the instance sanitises.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Whether new follows need approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    /// Whether the account is marked as automated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<bool>,
    /// Whether the account may appear in discovery.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<bool>,
    /// The profile picture, as the id of an image already uploaded to the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// The banner, as the id of an image already uploaded to the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    /// The profile's metadata rows, when the change replaces them.
    ///
    /// A save replaces the whole list: an empty list removes every row, and leaving this
    /// unset leaves the rows alone. The server trims every name and value, skips a row
    /// with both empty, and refuses with a 422 a list longer than six rows, a blank name
    /// beside a non-blank value, markup, or two names equal once trimmed and lowercased;
    /// the error names the row by its 1-based position. The rows travel in a JSON body
    /// because the endpoint's form encoding cannot spell a nested list, so a change that
    /// also uploads an image uses multipart and does not send the rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<ProfileField>>,
    /// The profile picture, to upload with this change.
    #[serde(skip)]
    pub avatar_image: Option<ProfileImage>,
    /// The banner, to upload with this change.
    #[serde(skip)]
    pub header_image: Option<ProfileImage>,
}

/// One row of the profile's name/value table.
///
/// The instance stores the trimmed pair and reports validation failures against the
/// row's 1-based position, so a caller that validates first can point at the same row.
///
/// The same row shape is read from an account and written back in a profile update: the
/// read is the only side that carries [`ProfileField::verified_at`], because the instance
/// mints that proof and no client can assert it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileField {
    /// Label shown beside the value, e.g. `Pronouns`.
    pub name: String,
    /// The value, usually a link or a short line of text.
    pub value: String,
    /// When the instance last verified that the value is a link the account controls.
    ///
    /// `null` for a row that was never verified or whose proof lapsed. A write leaves
    /// this unset: the instance decides verification, so echoing a previously read
    /// value would send a claim the account cannot make.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
}

impl ProfileField {
    /// A row with a label and its value.
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            verified_at: None,
        }
    }

    /// When the instance verified the value; see [`ProfileField::verified_at`].
    #[must_use]
    pub fn verified_at(mut self, verified_at: impl Into<String>) -> Self {
        self.verified_at = Some(verified_at.into());
        self
    }
}

/// An image sent with a profile change.
///
/// The bytes travel with the request rather than through the media endpoint first:
/// the profile endpoints take `multipart/form-data`, and an image uploaded there is
/// cropped and stored as an avatar, not as a post attachment.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ProfileImage {
    pub bytes: Vec<u8>,
    pub mime: String,
}

impl ProfileImage {
    /// An image of `bytes` with the MIME type its filename extension already implies.
    #[must_use]
    pub fn new(bytes: impl Into<Vec<u8>>, mime: impl Into<String>) -> Self {
        Self {
            bytes: bytes.into(),
            mime: mime.into(),
        }
    }
}

impl core::fmt::Debug for ProfileImage {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ProfileImage")
            .field("mime", &self.mime)
            .field("bytes", &format_args!("{} bytes", self.bytes.len()))
            .finish()
    }
}

impl ProfileUpdate {
    /// An update that changes nothing, to build on.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the display name.
    #[must_use]
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Sets the bio.
    #[must_use]
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Requires approval for new follows.
    #[must_use]
    pub const fn locked(mut self, locked: bool) -> Self {
        self.locked = Some(locked);
        self
    }

    /// Sets the profile picture from an uploaded image.
    #[must_use]
    pub fn avatar(mut self, media_id: impl Into<String>) -> Self {
        self.avatar = Some(media_id.into());
        self
    }

    /// Sets the banner from an uploaded image.
    #[must_use]
    pub fn header(mut self, media_id: impl Into<String>) -> Self {
        self.header = Some(media_id.into());
        self
    }

    /// Uploads a new profile picture with this change.
    #[must_use]
    pub fn avatar_image(mut self, image: ProfileImage) -> Self {
        self.avatar_image = Some(image);
        self
    }

    /// Uploads a new banner with this change.
    #[must_use]
    pub fn header_image(mut self, image: ProfileImage) -> Self {
        self.header_image = Some(image);
        self
    }

    /// Replaces the profile's metadata rows; see [`ProfileUpdate::fields`].
    #[must_use]
    pub fn fields(mut self, fields: Vec<ProfileField>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Whether the result says anything at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.display_name.is_none()
            && self.note.is_none()
            && self.locked.is_none()
            && self.bot.is_none()
            && self.discoverable.is_none()
            && self.avatar.is_none()
            && self.header.is_none()
            && self.fields.is_none()
            && self.avatar_image.is_none()
            && self.header_image.is_none()
    }
}
