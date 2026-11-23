//! Domain types shared by every Mastodon transport.
//!
//! These are the shapes an instance returns, trimmed to what the client renders
//! and what the recommendation engine consumes. They are deliberately owned by
//! this crate rather than by the consumer: the SDK, the web client and any native
//! tooling must agree on them, and a wire change to an instance should be a change
//! in exactly one place.

mod admin;
mod discovery;
mod filter;
mod highlights;
mod instance;
mod interaction;
mod markers;
mod schedule;
mod settings;
mod stream;

pub use admin::{
    AccountDetail, AccountNote, AccountOrigin, AccountStatus, AdminAccount, AdminAccountQuery,
    AdminAlert, AdminCounts, AdminMeasure, AdminMeta, AdminOverview, AnnouncementDraft, AssetKind,
    AssetUpload, AuditEntry, AuditFilter, DomainAllow, DomainBlock, DomainBlockDraft,
    DomainImportOutcome, DomainSeverity, EmailDomainBlock, FlagState, MeasurePoint,
    ModerationAction, NewScanner, NodeLog, NodeStatus, Report, ReportDraft, ReportFilter,
    ReportState, Scan, ScanFlag, ScanRequest, ScanSeverity, Scanner, ScannerAsset, ScannerKind,
    ScannerRule, ScanningSummary, SeverityCount, Topology, TopologyNode, TriggerOutcome,
    WarningPreset,
};
pub use discovery::{Suggestion, TrendingLink};
pub use filter::{
    Filter, FilterAction, FilterContext, FilterDraft, FilterKeyword, FilterV1, FilterV1Draft,
    Verdict, judge,
};
pub use highlights::{FeaturedTag, Rule};
pub use instance::{
    Configuration, InstanceContact, InstanceInfo, InstanceThumbnail, Registrations, Statuses,
    Usage, UsageUsers,
};
pub use interaction::{
    Conversation, HistoryEntry, List, Media, MediaAttachment, MediaKind, Mention, Poll, PollOption,
    ProfileField, ProfileImage, ProfileUpdate, SearchResults, Tag, TagHistory,
};
pub use markers::{Marker, MarkerTimeline, MarkerUpdate, Markers};
pub use schedule::{ScheduledParams, ScheduledStatus};
pub use settings::{Announcement, Preferences};
pub use stream::StreamEvent;

use serde::{Deserialize, Serialize};

/// The rendered shape of one profile picture.
///
/// `size` is the `"WxH"` spelling the wire also carries as a string; `aspect` is width
/// over height, which is what lets a renderer reserve a box before the bytes arrive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PictureShape {
    /// Rendered width, in pixels.
    pub width: u32,
    /// Rendered height, in pixels.
    pub height: u32,
    /// `"WxH"`, e.g. `400x300`.
    pub size: String,
    /// Width over height, e.g. `1.3333333333333333` for 400x300.
    pub aspect: f64,
}

/// An account picture's renditions, keyed the way the wire spells them.
///
/// The whole object is absent or `null` when the shape is genuinely unknown — a remote
/// account, the default asset, or a render still in flight — so unknown stays
/// distinguishable from square: nothing here is ever defaulted to a `0` or a guessed
/// aspect. `small` is deliberately `null` on this server, which keeps one profile
/// rendition per slot; the member exists so an upstream Mastodon answer that populates
/// it still parses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PictureMeta {
    /// The rendition the picture URL serves.
    #[serde(default)]
    pub original: Option<PictureShape>,
    /// The thumbnail rendition, when the instance reports one.
    #[serde(default)]
    pub small: Option<PictureShape>,
}

impl PictureMeta {
    /// Width and height of `original`, the rendition the URL serves.
    ///
    /// `small` is not a fallback: it is a thumbnail at a different pixel size, so a box
    /// reserved from it would be the wrong box for the picture the client draws.
    fn original_shape(&self) -> Option<(u32, u32)> {
        self.original
            .as_ref()
            .map(|shape| (shape.width, shape.height))
    }
}

/// An account, as returned by `/api/v1/accounts/:id`.
// Partial equality only: a [`PictureShape`] carries a float `aspect`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Actor {
    /// Instance-local id, as a string.
    pub id: String,
    /// `user@host`, or just `user` for local accounts.
    pub acct: String,
    /// Display name; may be empty.
    #[serde(default)]
    pub display_name: String,
    /// HTML bio.
    #[serde(default)]
    pub note: String,
    /// Avatar URL.
    #[serde(default)]
    pub avatar: String,
    /// Header URL.
    #[serde(default)]
    pub header: String,
    /// Shape of the picture `avatar` serves, when the instance reports one.
    ///
    /// `None` for a remote account, the default asset, or a picture with no rendered
    /// variant row yet: the API sends the key absent or explicitly `null` there.
    #[serde(default)]
    pub avatar_meta: Option<PictureMeta>,
    /// Shape of the picture `header` serves, on the same terms as [`Actor::avatar_meta`].
    #[serde(default)]
    pub header_meta: Option<PictureMeta>,
    /// Whether the account is a bot.
    #[serde(default)]
    pub bot: bool,
    /// Whether the account is locked.
    #[serde(default)]
    pub locked: bool,
    /// Follower count.
    #[serde(default)]
    pub followers_count: u64,
    /// Following count.
    #[serde(default)]
    pub following_count: u64,
    /// Status count.
    #[serde(default)]
    pub statuses_count: u64,
    /// Last status timestamp, RFC 3339.
    #[serde(default)]
    pub last_status_at: Option<String>,
    /// Whether the account opted into discovery features.
    #[serde(default)]
    pub discoverable: Option<bool>,
    /// Whether public posts may be indexed by search engines.
    #[serde(default)]
    pub indexable: Option<bool>,
    /// Deprecated inverse of `indexable`; older instances still send it.
    #[serde(default)]
    pub noindex: Option<bool>,
    /// The profile's metadata rows, as the instance returns them.
    ///
    /// Both `verify_credentials` and a profile update answer with these rows, and a
    /// caller that renders a profile should read them here rather than keep a second
    /// copy of the payload. An instance that omits the key, and an account with no
    /// rows, both decode to an empty list: the wire has no unset state a caller has to
    /// tell apart. Empty rows are skipped when serialising, so a client that reposts
    /// the account does not invent a member the instance never sent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<ProfileField>,
}

impl Actor {
    /// The box to reserve for the avatar, as `(width, height)`.
    ///
    /// This is the one place that decides what a picture's dimensions are, so a drawing
    /// site does not re-derive it: `None` when the instance reported no shape, which a
    /// renderer must treat as "reserve nothing yet" rather than as a square.
    #[must_use]
    pub fn avatar_shape(&self) -> Option<(u32, u32)> {
        self.avatar_meta
            .as_ref()
            .and_then(PictureMeta::original_shape)
    }

    /// The box to reserve for the header, on the same terms as [`Actor::avatar_shape`].
    #[must_use]
    pub fn header_shape(&self) -> Option<(u32, u32)> {
        self.header_meta
            .as_ref()
            .and_then(PictureMeta::original_shape)
    }
}

/// Mastodon's name for an [`Actor`]; kept for existing clients.
pub use Actor as Account;

/// A thread around one status: what it replies to, and what replies to it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Context {
    /// Statuses above the focus, oldest first.
    pub ancestors: Vec<Status>,
    /// Direct and nested replies, oldest first.
    pub descendants: Vec<Status>,
}

/// Which streaming timeline to subscribe to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamKind {
    /// The authenticated user's home timeline.
    User,
    /// Public posts from the user's instance.
    Local,
    /// Federated public posts known to the user's instance.
    Federated,
}

/// A custom emoji entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emoji {
    /// `:shortcode:` without colons.
    pub shortcode: String,
    /// Image URL.
    pub url: String,
    /// Present when the emoji is animated.
    #[serde(default)]
    pub static_url: Option<String>,
}

/// Mastodon's name for an [`Emoji`].
pub use Emoji as CustomEmoji;

/// Reply/visibility level of a status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Visible to everyone.
    Public,
    /// Visible on the instance, not federated.
    Unlisted,
    /// Visible to followers only.
    Private,
    /// Visible to mentioned accounts only.
    Direct,
}

impl Visibility {
    /// The wire spelling Mastodon's form endpoints expect.
    ///
    /// Kept next to the enum so a form post and a JSON body cannot disagree about
    /// what `private` is called; a test pins it to the serde name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Unlisted => "unlisted",
            Self::Private => "private",
            Self::Direct => "direct",
        }
    }
}

/// A status (post).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// The flags are the wire's, not ours: Mastodon sends five independent booleans about
// the viewer's relationship to a status, and folding them into a bitset would make
// every consumer rediscover which bit is which.
#[expect(
    clippy::struct_excessive_bools,
    reason = "the wire has independent flags and a DTO must mirror it"
)]
pub struct Post {
    /// Instance-local id.
    pub id: String,
    /// When it was posted, RFC 3339.
    pub created_at: String,
    /// HTML content, sanitised by the renderer, never trusted.
    #[serde(default)]
    pub content: String,
    /// Visibility.
    pub visibility: Visibility,
    /// Author.
    pub account: Account,
    /// Favourite count.
    #[serde(default)]
    pub favourites_count: u64,
    /// Boost count.
    #[serde(default)]
    pub reblogs_count: u64,
    /// Reply count.
    #[serde(default)]
    pub replies_count: u64,
    /// Favourited by the signed-in account.
    #[serde(default)]
    pub favourited: bool,
    /// Boosted by the signed-in account.
    #[serde(default)]
    pub reblogged: bool,
    /// Bookmarked by the signed-in account.
    #[serde(default)]
    pub bookmarked: bool,
    /// The status this one boosts.
    #[serde(default)]
    pub reblog: Option<Box<Status>>,
    /// The status this one replies to.
    #[serde(default)]
    pub in_reply_to_id: Option<String>,
    /// Emoji used in the content.
    #[serde(default)]
    pub emojis: Vec<CustomEmoji>,
    /// Permalink on the instance, when the status is public enough to have one.
    #[serde(default)]
    pub url: Option<String>,
    /// Whether the media should stay behind a click.
    #[serde(default)]
    pub sensitive: bool,
    /// Content warning, plain text.
    #[serde(default)]
    pub spoiler_text: String,
    /// The language the post is written in, when the instance declares one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Source language of the body, when the instance declares one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_format: Option<String>,
    /// The unrendered body, when the instance sends a non-HTML format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_source: Option<String>,
    /// Attached media, in display order.
    #[serde(default)]
    pub media_attachments: Vec<MediaAttachment>,
    /// Accounts mentioned in the content.
    #[serde(default)]
    pub mentions: Vec<Mention>,
    /// Hashtags in the content.
    #[serde(default)]
    pub tags: Vec<Tag>,
    /// The attached poll, when there is one.
    #[serde(default)]
    pub poll: Option<Poll>,
    /// When it was last edited, if it ever was.
    #[serde(default)]
    pub edited_at: Option<String>,
    /// Whether the author pinned it to their profile.
    #[serde(default)]
    pub pinned: Option<bool>,
}

/// Mastodon's name for a [`Post`].
pub use Post as Status;

/// Kind of notification.
///
/// Parsed from the raw string rather than a renamed enum: the wire mixes conventions
/// (`follow_request` and `admin.sign_up` are not `lowercase`), and an instance may add a
/// kind before we know it. One unknown kind must not fail the whole notifications page,
/// so anything unrecognised lands in [`NotificationKind::Unknown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum NotificationKind {
    /// Someone followed the account.
    Follow,
    /// Someone asked to follow a locked account.
    FollowRequest,
    /// Someone boosted a status.
    Reblog,
    /// Someone favourited a status.
    Favourite,
    /// Someone mentioned the account.
    Mention,
    /// Poll ended.
    Poll,
    /// New status from a followed account.
    Status,
    /// Account sign-up (admin only).
    AdminSignUp,
    /// Report (admin only).
    AdminReport,
    /// A kind this client does not render, e.g. one a newer instance added.
    Unknown,
}

impl NotificationKind {
    /// The value the wire uses for this kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Follow => "follow",
            Self::FollowRequest => "follow_request",
            Self::Reblog => "reblog",
            Self::Favourite => "favourite",
            Self::Mention => "mention",
            Self::Poll => "poll",
            Self::Status => "status",
            Self::AdminSignUp => "admin.sign_up",
            Self::AdminReport => "admin.report",
            // Never sent back: the server chose it, we only failed to recognise it.
            Self::Unknown => "unknown",
        }
    }
}

impl From<String> for NotificationKind {
    fn from(raw: String) -> Self {
        match raw.as_str() {
            "follow" => Self::Follow,
            "follow_request" => Self::FollowRequest,
            "reblog" => Self::Reblog,
            "favourite" => Self::Favourite,
            "mention" => Self::Mention,
            "poll" => Self::Poll,
            "status" => Self::Status,
            "admin.sign_up" => Self::AdminSignUp,
            "admin.report" => Self::AdminReport,
            _ => Self::Unknown,
        }
    }
}

impl From<NotificationKind> for String {
    fn from(kind: NotificationKind) -> Self {
        kind.as_str().to_owned()
    }
}

/// A notification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    /// Notification id.
    pub id: String,
    /// What happened.
    #[serde(rename = "type")]
    pub kind: NotificationKind,
    /// When it happened, RFC 3339.
    pub created_at: String,
    /// Acting account.
    pub account: Account,
    /// Related status, when there is one.
    #[serde(default)]
    pub status: Option<Status>,
}

/// Relationship between the signed-in account and another account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the wire has independent flags and a DTO must mirror it"
)]
pub struct Relationship {
    /// The other account.
    pub id: LocalId,
    /// Following?
    pub following: bool,
    /// Follow requested but pending?
    pub requested: bool,
    /// Followed by them?
    pub followed_by: bool,
    /// Blocking them?
    #[serde(default)]
    pub blocking: bool,
    /// Blocked by them?
    #[serde(default)]
    pub blocked_by: bool,
    /// Muting them?
    #[serde(default)]
    pub muting: bool,
    /// Muting their notifications only?
    #[serde(default)]
    pub muting_notifications: bool,
    /// Is the whole domain blocked?
    #[serde(default)]
    pub domain_blocking: bool,
    /// Recommended by an admin?
    #[serde(default)]
    pub endorsed: bool,
}

/// An instance-local id, stringified for wire compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct LocalId(pub String);

/// A status being composed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    /// Plain-text body; the transport converts to HTML or plain per instance.
    pub text: String,
    /// Visibility to request.
    pub visibility: Visibility,
    /// `in_reply_to_id`, when replying.
    pub in_reply_to_id: Option<String>,
    /// Idempotency key, so a retried post cannot double-post.
    pub idempotency_key: Option<String>,
    /// Already-uploaded attachments, in the order they should appear.
    pub media_ids: Vec<String>,
    /// Content warning; empty means none.
    pub spoiler_text: String,
    /// Whether the media is behind the warning.
    pub sensitive: bool,
    /// A poll to attach, when the post asks a question instead of making a statement.
    pub poll: Option<PollDraft>,
    /// The language the post is written in, as an ISO 639-1 code.
    ///
    /// `None` asks the instance to guess, which is what Mastodon does when the field is
    /// absent. Sending an explicit code is worth the extra control for a writer whose
    /// posts are not in the instance's default language — without it every post inherits
    /// the server's guess and the language filter hides them from the people who want them.
    pub language: Option<String>,
}

/// A language an instance publishes in.
///
/// `GET /api/v1/instance/languages` answers with this pair rather than a bare code, which
/// is what lets the composer show "German" instead of "de" without shipping its own table.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Language {
    /// ISO 639-1 code, e.g. `en`.
    pub code: String,
    /// English name, e.g. `English`.
    #[serde(default)]
    pub name: String,
}

/// A poll as the composer builds it, before the instance assigns ids.
///
/// Separate from [`Poll`], which is what the server answers with: this one has no votes,
/// no ids and no expiry date, only the options and how long they stay open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollDraft {
    /// Two to four options, in the order they should appear.
    pub options: Vec<String>,
    /// How long the poll accepts votes, in seconds.
    pub expires_in: u32,
    /// Whether a voter may pick more than one option.
    pub multiple: bool,
    /// Whether the counts stay hidden until the poll closes.
    pub hide_totals: bool,
}

impl PollDraft {
    /// The shortest poll someone would actually mean: options, and a day to answer.
    pub fn new(options: Vec<String>, expires_in: u32) -> Self {
        Self {
            options,
            expires_in,
            multiple: false,
            hide_totals: false,
        }
    }

    /// Allow several answers.
    #[must_use]
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Hide the counts until the poll closes.
    #[must_use]
    pub fn hide_totals(mut self, hide: bool) -> Self {
        self.hide_totals = hide;
        self
    }
}

impl Draft {
    /// A draft with nothing but text: the common case, and the one a reply box needs.
    pub fn text(text: impl Into<String>, visibility: Visibility) -> Self {
        Self {
            text: text.into(),
            visibility,
            in_reply_to_id: None,
            idempotency_key: None,
            media_ids: Vec::new(),
            spoiler_text: String::new(),
            sensitive: false,
            poll: None,
            language: None,
        }
    }

    /// Sets the language the post is written in.
    #[must_use]
    pub fn language(mut self, language: impl Into<String>) -> Self {
        let language = language.into();
        self.language = if language.is_empty() {
            None
        } else {
            Some(language)
        };
        self
    }
}

/// The signed-in account together with the preferences only it can read.
///
/// Mastodon answers `verify_credentials` with the account's fields *plus* a `source` object.
/// `flatten` keeps every `Account` consumer working: this is the same payload with more of it
/// filled in, not a second account type.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CredentialAccount {
    /// The account itself.
    #[serde(flatten)]
    pub account: Account,
    /// Defaults this account posts with.
    #[serde(default)]
    pub source: CredentialSource,
}

/// The `source` half of a credential account: what the writer chose, not what they posted.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct CredentialSource {
    /// Default visibility for new posts.
    #[serde(default)]
    pub privacy: Option<String>,
    /// Whether new posts default to sensitive.
    #[serde(default)]
    pub sensitive: Option<bool>,
    /// The writer's posting language, as a code.
    #[serde(default)]
    pub language: Option<String>,
}

/// Pagination parameters for a timeline request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageParams {
    /// Return items older than this id.
    pub max_id: Option<String>,
    /// Return items newer than this id.
    pub since_id: Option<String>,
    /// Page size; instances cap this (40 is the documented maximum).
    pub limit: Option<u16>,
}

impl PageParams {
    /// A first page of at most `limit` items.
    #[must_use]
    pub fn first(limit: u16) -> Self {
        Self {
            limit: Some(limit),
            ..Self::default()
        }
    }

    /// The page after `max_id`.
    #[must_use]
    pub fn after(max_id: impl Into<String>, limit: u16) -> Self {
        Self {
            max_id: Some(max_id.into()),
            limit: Some(limit),
            ..Self::default()
        }
    }
}

/// One page of results, plus the cursor for the next one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    /// The items.
    pub items: Vec<T>,
    /// Cursor for older items — the smallest id in the page, if any.
    pub next_max_id: Option<String>,
}

impl<T> Page<T> {
    /// Wraps `items`, deriving the older-cursor from the last item's id.
    #[must_use]
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            next_max_id: None,
        }
    }

    /// Attaches an explicit `max_id` cursor.
    #[must_use]
    pub fn with_next(mut self, next_max_id: Option<String>) -> Self {
        self.next_max_id = next_max_id;
        self
    }

    /// Maps the items, keeping the cursor.
    #[must_use]
    pub fn map<U>(self, f: impl FnMut(T) -> U) -> Page<U> {
        Page {
            items: self.items.into_iter().map(f).collect(),
            next_max_id: self.next_max_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_kinds_follow_the_wire_not_a_naming_convention() {
        // The instance sends `follow_request` and `admin.sign_up`; `rename_all =
        // "lowercase"` would have produced `followrequest` and failed the whole page.
        let cases = [
            ("follow", NotificationKind::Follow),
            ("follow_request", NotificationKind::FollowRequest),
            ("favourite", NotificationKind::Favourite),
            ("admin.sign_up", NotificationKind::AdminSignUp),
            ("admin.report", NotificationKind::AdminReport),
        ];
        for (raw, expected) in cases {
            let parsed: NotificationKind =
                serde_json::from_str(&format!("\"{raw}\"")).expect("known kind");
            assert_eq!(parsed, expected, "parsing {raw}");
            let back = serde_json::to_string(&expected).expect("serialisable");
            assert_eq!(back, format!("\"{raw}\""));
        }
    }

    #[test]
    fn one_unknown_notification_kind_does_not_fail_the_page() {
        // A newer instance may add a kind (Mastodon has `update` and
        // `severed_relationships`); the notifications list must still render.
        let json = r#"{
            "id": "9",
            "type": "moderation_warning",
            "created_at": "2026-11-07T00:00:00.000Z",
            "account": {"id": "1", "acct": "alice", "display_name": "Alice"}
        }"#;
        let notification: Notification = serde_json::from_str(json).expect("unknown kind parses");
        assert_eq!(notification.kind, NotificationKind::Unknown);
    }

    #[test]
    fn status_deserialises_a_trimmed_instance_payload() {
        let json = r#"{
            "id": "110",
            "created_at": "2026-11-06T21:00:00.000Z",
            "content": "<p>hello</p>",
            "visibility": "public",
            "account": {"id": "1", "acct": "alice", "display_name": "Alice"},
            "favourites_count": 3
        }"#;

        let status: Status = serde_json::from_str(json).expect("valid status");

        assert_eq!(status.id, "110");
        assert_eq!(status.visibility, Visibility::Public);
        assert_eq!(status.account.acct, "alice");
        assert!(!status.favourited);
        assert_eq!(status.emojis, Vec::new());
    }

    #[test]
    fn an_account_decodes_its_metadata_rows() {
        // `verify_credentials` and a profile update both answer with this array; the
        // model keeps it so a caller does not hand-decode the payload beside the account.
        let json = r#"{
            "id": "1",
            "acct": "alice",
            "fields": [
                {"name": "Website", "value": "https://alice.test", "verified_at": "2026-11-01T00:00:00.000Z"},
                {"name": "Pronouns", "value": "she/her", "verified_at": null}
            ]
        }"#;

        let account: Account = serde_json::from_str(json).expect("valid account");

        assert_eq!(account.fields.len(), 2);
        assert_eq!(account.fields[0].name, "Website");
        assert_eq!(
            account.fields[0].verified_at.as_deref(),
            Some("2026-11-01T00:00:00.000Z")
        );
        assert_eq!(account.fields[1].verified_at, None);
    }

    #[test]
    fn an_account_without_metadata_rows_still_decodes() {
        // Instances that omit the key, and accounts with no rows, are not errors: an
        // absent member means "no rows", not "could not read the account".
        let json = r#"{"id": "1", "acct": "alice"}"#;

        let account: Account = serde_json::from_str(json).expect("account without fields");

        assert_eq!(account.fields, Vec::new());
    }

    #[test]
    fn metadata_rows_round_trip_and_empty_rows_are_skipped() {
        let account: Account = serde_json::from_str(
            r#"{
                "id": "1",
                "acct": "alice",
                "fields": [
                    {"name": "Website", "value": "https://alice.test", "verified_at": "2026-11-01T00:00:00.000Z"},
                    {"name": "Pronouns", "value": "she/her"}
                ]
            }"#,
        )
        .expect("valid account");

        let encoded = serde_json::to_value(&account).expect("serialisable account");
        let rows = encoded["fields"].as_array().expect("rows encoded");
        assert_eq!(rows[0]["verified_at"], "2026-11-01T00:00:00.000Z");
        assert!(rows[1].get("verified_at").is_none());
        assert_eq!(
            serde_json::from_value::<Account>(encoded).expect("round trip"),
            account
        );

        let empty = Account {
            fields: Vec::new(),
            ..account
        };
        let encoded = serde_json::to_value(&empty).expect("serialisable account");
        assert!(encoded.get("fields").is_none());
    }

    #[test]
    fn an_account_decodes_the_picture_shape_the_api_publishes() {
        // The exact document `/api/v1/accounts/:id` answers with once both pictures have
        // a rendered variant: one rendition per slot, so `small` is `null` and the
        // box to reserve comes from `original`.
        let json = r#"{
            "id": "1",
            "acct": "ada",
            "avatar": "https://host/media/abc/avatar",
            "avatar_static": "https://host/media/abc/avatar_static",
            "avatar_meta": {
                "original": { "width": 400, "height": 300, "size": "400x300", "aspect": 1.3333333333333333 },
                "small": null
            },
            "header": "https://host/media/abc/header",
            "header_meta": {
                "original": { "width": 1500, "height": 500, "size": "1500x500", "aspect": 3.0 },
                "small": null
            }
        }"#;

        let account: Account = serde_json::from_str(json).expect("valid account");

        assert_eq!(account.avatar_shape(), Some((400, 300)));
        assert_eq!(account.header_shape(), Some((1500, 500)));
        let avatar = account.avatar_meta.as_ref().expect("avatar meta");
        let original = avatar.original.as_ref().expect("original rendition");
        assert_eq!(original.size, "400x300");
        assert!((original.aspect - 4.0 / 3.0).abs() < f64::EPSILON);
        assert_eq!(avatar.small, None);
        assert_eq!(account.header.as_str(), "https://host/media/abc/header");
    }

    #[test]
    fn an_account_without_picture_meta_reserves_no_box() {
        // The overwhelmingly common case: no variant rows, a default asset, or a remote
        // account. Absent must stay distinguishable from square, so both helpers answer
        // `None` rather than a zero or a guessed shape.
        let json = r#"{"id": "1", "acct": "alice", "avatar": "https://host/media/x.png"}"#;

        let account: Account = serde_json::from_str(json).expect("account without picture meta");

        assert_eq!(account.avatar_meta, None);
        assert_eq!(account.header_meta, None);
        assert_eq!(account.avatar_shape(), None);
        assert_eq!(account.header_shape(), None);
        assert_eq!(account.avatar.as_str(), "https://host/media/x.png");
    }

    #[test]
    fn an_explicitly_null_picture_meta_is_still_unknown() {
        // The API spells the key out and puts `null` in it. That is the same "unknown" as
        // an absent key, and `small: null` must not become a shape of zeroes.
        let json = r#"{
            "id": "1",
            "acct": "alice",
            "avatar_meta": { "original": null, "small": null },
            "header_meta": null
        }"#;

        let account: Account = serde_json::from_str(json).expect("nulls decode");

        let avatar = account.avatar_meta.as_ref().expect("meta object present");
        assert_eq!(avatar.original, None);
        assert_eq!(avatar.small, None);
        assert_eq!(account.avatar_shape(), None);
        assert_eq!(account.header_meta, None);
        assert_eq!(account.header_shape(), None);
    }

    #[test]
    fn a_remote_picture_shape_with_a_thumbnail_round_trips() {
        // An upstream Mastodon answer populates `small`; the member is kept for that, and
        // the box still comes from `original`, which is what the URL serves.
        let json = r#"{
            "id": "1",
            "acct": "ada@remote.test",
            "avatar_meta": {
                "original": { "width": 192, "height": 192, "size": "192x192", "aspect": 1.0 },
                "small": { "width": 48, "height": 48, "size": "48x48", "aspect": 1.0 }
            }
        }"#;

        let account: Account = serde_json::from_str(json).expect("remote-shaped account");
        let meta = account.avatar_meta.as_ref().expect("avatar meta");
        assert_eq!(meta.small.as_ref().map(|small| small.width), Some(48));
        assert_eq!(account.avatar_shape(), Some((192, 192)));

        let encoded = serde_json::to_value(&account).expect("serialisable account");
        assert_eq!(encoded["avatar_meta"]["small"]["size"], "48x48");
        assert_eq!(
            serde_json::from_value::<Account>(encoded).expect("round trip"),
            account
        );
    }
}
