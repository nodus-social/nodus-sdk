//! What a moderator sees and can do.
//!
//! These types are `admin:read`/`admin:write` territory: they carry private data — a
//! report's free-text comment, an account's email, the private note on a domain block —
//! so nothing here is used by a public screen, and every call that returns them is
//! expected to fail for a token that is not a moderator's.

use serde::{Deserialize, Serialize};

use super::{Account, Status};

/// A report, as the queue returns it.
///
/// The queue hydrates everything triage needs — the reporter, the reported account, the
/// posts being complained about, and whoever claimed the case — so a moderator reads one
/// response rather than one per row.
///
/// No `Eq`: the statuses it quotes carry floats, and a report is compared only in tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    /// `open` or `resolved`. Kept as a string because a newer server may add a state and
    /// one unfamiliar value must not fail the whole queue.
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub action_taken: bool,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub comment: String,
    /// Rules the reporter says were broken.
    #[serde(default)]
    pub rule_ids: Vec<String>,
    /// The moderator's reason for the decision, empty until one was made.
    #[serde(default)]
    pub action_taken_note: String,
    #[serde(default)]
    pub action_taken_at: Option<String>,
    /// The moderator who decided, when the server still knows the account.
    #[serde(default)]
    pub action_taken_by: Option<Account>,
    /// Who claimed the case; `None` is the unclaimed queue.
    #[serde(default)]
    pub assigned_account_id: Option<String>,
    #[serde(default)]
    pub assigned_account: Option<Account>,
    /// Who filed it. Absent on servers that strip the reporter from the queue.
    #[serde(default)]
    pub reporter: Option<Account>,
    #[serde(default)]
    pub created_at: String,
    /// When the report last changed, so a queue can show what moved.
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub target_account: Option<Account>,
    #[serde(default)]
    pub statuses: Vec<Status>,
}

/// Which reports to ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportState {
    /// Waiting for a moderator.
    #[default]
    Open,
    /// Already acted on.
    Resolved,
}

impl ReportState {
    /// The query value the API expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Resolved => "resolved",
        }
    }
}

/// An account as a moderator sees it: the public account plus the private fields.
///
/// The flags are separate booleans because the wire has separate booleans and each one
/// means something different to a moderator; folding them into a state enum would make
/// a payload with two flags set unrepresentable, and such payloads do occur.
#[expect(
    clippy::struct_excessive_bools,
    reason = "the wire has independent flags and a DTO must mirror it"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminAccount {
    pub id: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub confirmed: bool,
    #[serde(default)]
    pub silenced: bool,
    #[serde(default)]
    pub suspended: bool,
    #[serde(default)]
    pub disabled: bool,
    /// Missing means the server has no approval concept at all (before Mastodon 4.0),
    /// which reads as approved — defaulting to `false` would show every account as
    /// pending on those servers.
    #[serde(default = "approved_by_default")]
    pub approved: bool,
    /// Whether this account holds administrator rights.
    #[serde(default)]
    pub admin: bool,
    /// Reports against this account that nobody has resolved yet.
    #[serde(default)]
    pub open_reports: i64,
    /// How many moderator notes this account carries.
    #[serde(default)]
    pub notes_count: i64,
    /// Whether this account's images are stored at full size rather than resized.
    #[serde(default)]
    pub lossless_images: bool,
    /// Whether this account's video is stored at full size rather than resized.
    #[serde(default)]
    pub lossless_videos: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub account: Option<Account>,
}

impl AdminAccount {
    /// What this account is, in one word, for a table cell.
    pub fn standing(&self) -> &'static str {
        if self.suspended {
            "suspended"
        } else if self.silenced {
            "silenced"
        } else if self.disabled {
            "disabled"
        } else if !self.approved {
            "pending"
        } else if self.confirmed {
            "confirmed"
        } else {
            "unconfirmed"
        }
    }
}

/// The default for a field an older server does not send.
const fn approved_by_default() -> bool {
    true
}

/// What a moderator decided about an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModerationAction {
    /// Limit the account to its followers.
    Silence,
    /// Stop the account from being seen or heard at all.
    Suspend,
    /// Refuse the account entirely.
    Disable,
    /// Undo silence, suspension or disabling.
    Enable,
    /// Accept a pending account.
    Approve,
    /// Reject a pending account.
    Reject,
}

impl ModerationAction {
    /// The action name the API expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Silence => "silence",
            Self::Suspend => "suspend",
            Self::Disable => "disable",
            Self::Enable => "enable",
            Self::Approve => "approve",
            Self::Reject => "reject",
        }
    }

    /// Every action, for a menu.
    pub const ALL: [Self; 6] = [
        Self::Silence,
        Self::Suspend,
        Self::Disable,
        Self::Enable,
        Self::Approve,
        Self::Reject,
    ];
}

/// How much of a domain to refuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DomainSeverity {
    /// Refuse everything, including existing posts.
    #[default]
    Suspend,
    /// Refuse the domain's public posts from timelines, keep existing ones.
    Silence,
    /// Keep the block on record without refusing anything.
    Noop,
}

impl DomainSeverity {
    /// The severity name the API expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Suspend => "suspend",
            Self::Silence => "silence",
            Self::Noop => "noop",
        }
    }
}

/// A blocked domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainBlock {
    pub id: String,
    pub domain: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub severity: DomainSeverity,
    #[serde(default)]
    pub reject_media: bool,
    #[serde(default)]
    pub reject_reports: bool,
    #[serde(default)]
    pub obfuscate: bool,
    #[serde(default)]
    pub public_comment: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_account_without_a_domain_is_local() {
        let account: AdminAccount = serde_json::from_value(serde_json::json!({
            "id": "1",
            "username": "ada",
            "confirmed": true,
            "approved": true,
        }))
        .unwrap();
        assert_eq!(account.domain, None);
        assert_eq!(account.standing(), "confirmed");
    }

    #[test]
    fn standing_reports_the_most_severe_flag() {
        let mut account: AdminAccount = serde_json::from_value(serde_json::json!({
            "id": "1",
            "username": "ada",
        }))
        .unwrap();
        assert_eq!(account.standing(), "unconfirmed");
        account.suspended = true;
        assert_eq!(account.standing(), "suspended");
    }

    #[test]
    fn a_report_without_statuses_still_parses() {
        let report: Report = serde_json::from_value(serde_json::json!({
            "id": "9",
            "comment": "spam",
        }))
        .unwrap();
        assert_eq!(report.statuses, Vec::new());
        assert!(!report.action_taken);
    }

    #[test]
    fn a_report_carries_its_statuses_and_handling() {
        let report: Report = serde_json::from_value(serde_json::json!({
            "id": "9",
            "state": "resolved",
            "category": "spam",
            "comment": "repeated promotion",
            "rule_ids": ["1", "4"],
            "action_taken": true,
            "action_taken_note": "silenced",
            "action_taken_at": "2024-05-01T10:00:00Z",
            "action_taken_by": { "id": "7", "acct": "mod" },
            "assigned_account_id": "7",
            "created_at": "2024-05-01T09:00:00Z",
            "updated_at": "2024-05-01T10:00:00Z",
            "reporter": { "id": "3", "acct": "ada" },
            "target_account": { "id": "4", "acct": "spam@elsewhere" },
            "statuses": [
                {
                    "id": "100",
                    "created_at": "2024-04-30T08:00:00Z",
                    "visibility": "public",
                    "account": { "id": "4", "acct": "spam@elsewhere" },
                    "content": "buy things"
                }
            ]
        }))
        .unwrap();
        assert_eq!(report.state, "resolved");
        assert_eq!(report.rule_ids, ["1", "4"]);
        assert_eq!(report.statuses.len(), 1);
        assert_eq!(report.statuses[0].id, "100");
        assert_eq!(
            report
                .action_taken_by
                .as_ref()
                .map(|account| account.acct.as_str()),
            Some("mod")
        );
        assert!(report.assigned_account.is_none());
    }

    #[test]
    fn an_account_detail_keeps_its_notes_and_flags() {
        let detail: AccountDetail = serde_json::from_str(
            r#"{
                "id": "4",
                "username": "spam",
                "domain": "elsewhere",
                "approved": true,
                "admin": false,
                "open_reports": 2,
                "notes_count": 1,
                "created_at": "2024-01-01T00:00:00Z",
                "account": { "id": "4", "acct": "spam@elsewhere" },
                "notes": [
                    {
                        "id": "n1",
                        "account_id": "4",
                        "author_id": "7",
                        "author_username": "mod",
                        "body": "warned",
                        "created_at": "2024-05-01T00:00:00Z"
                    }
                ],
                "flags": [
                    {
                        "id": "f1",
                        "reason": "nsfw",
                        "severity": "sensitive",
                        "labels": ["nsfw"],
                        "score": 0.93,
                        "state": "open",
                        "created_at": "2024-05-01T00:00:00Z"
                    }
                ],
                "reports_against": 1
            }"#,
        )
        .expect("the detail payload parses");
        assert_eq!(detail.notes.len(), 1);
        assert_eq!(detail.notes[0].body, "warned");
        assert_eq!(detail.account.notes_count, 1);
        assert_eq!(detail.account.username, "spam");
        assert_eq!(detail.reports_against, 1);
        assert_eq!(detail.flags.len(), 1);
        assert_eq!(detail.flags[0].severity, ScanSeverity::Sensitive);
        assert!(detail.flags[0].score.is_some());
        assert!(!detail.account.suspended);
    }

    #[test]
    fn a_scanner_keeps_the_rules_that_judge_it() {
        let scanner: Scanner = serde_json::from_value(serde_json::json!({
            "id": "s1",
            "name": "clip",
            "kind": "video",
            "endpoint": "https://scan.example",
            "enabled": true,
            "contract": 2,
            "labels": ["nsfw", "gore"],
            "manifest": { "nsfw": "float32" },
            "timeout_ms": 8000,
            "rules": [
                { "id": "r1", "label": "nsfw", "threshold": 0.8, "severity": "block" },
                { "id": "r2", "label": "gore", "threshold": 0.5, "severity": "flag" }
            ]
        }))
        .unwrap();
        assert_eq!(scanner.kind, ScannerKind::Video);
        assert_eq!(scanner.rules.len(), 2);
        assert_eq!(scanner.rules[0].severity, ScanSeverity::Block);
        assert!(scanner.asset_id.is_none());
    }

    #[test]
    fn a_flag_defaults_to_unjudged() {
        let flag: ScanFlag = serde_json::from_value(serde_json::json!({
            "id": "f1",
            "scan_id": "sc1",
            "reason": "nsfw",
            "labels": ["nsfw"],
            "severity": "flag",
        }))
        .unwrap();
        assert_eq!(flag.state, FlagState::Open);
        assert_eq!(flag.severity, ScanSeverity::Flag);
        assert!(flag.score.is_none());
        assert!(flag.account_id.is_none());
    }

    #[test]
    fn a_measure_accepts_a_missing_previous_total() {
        let measures: Vec<AdminMeasure> = serde_json::from_value(serde_json::json!([
            {
                "key": "new_users",
                "total": 12,
                "previous_total": null,
                "data": [{ "date": "2024-05-01", "value": 3 }]
            }
        ]))
        .unwrap();
        assert_eq!(measures[0].total.as_deref(), Some("12"));
        assert_eq!(measures[0].previous_total, None);
        assert_eq!(measures[0].data[0].value, "3");
    }
}

/// One recorded administrative action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Row id.
    pub id: String,
    /// Account that performed it, when the row has one.
    #[serde(default)]
    pub actor_id: Option<String>,
    /// Action name, e.g. `resolve_report`.
    pub action: String,
    /// Object type, e.g. `report` or `account`.
    pub target_type: String,
    /// Object id, when there is one.
    #[serde(default)]
    pub target_id: Option<String>,
    /// Extra recorded fields.
    #[serde(default)]
    pub detail: serde_json::Value,
    /// When it happened.
    #[serde(default)]
    pub created_at: String,
}

/// The fields staff can set on an announcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnouncementDraft {
    /// Message body.
    pub content: String,
    /// Whether readers should see it now.
    #[serde(default)]
    pub published: bool,
    /// Whether the dates cover whole days.
    #[serde(default)]
    pub all_day: bool,
    /// Optional start timestamp.
    #[serde(default)]
    pub starts_at: Option<String>,
    /// Optional end timestamp.
    #[serde(default)]
    pub ends_at: Option<String>,
}

impl AnnouncementDraft {
    /// A draft with just a message.
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            published: true,
            all_day: false,
            starts_at: None,
            ends_at: None,
        }
    }
}

/// One line of the admin dashboard: a count per day, with the previous period beside it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminMeasure {
    pub key: String,
    /// The API sends totals as strings, so they are kept that way rather than parsed.
    #[serde(
        default,
        deserialize_with = "count_or_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub total: Option<String>,
    /// The same span one period earlier. `null` when the earlier half was never recorded,
    /// which a chart reads as "no comparison" rather than as zero.
    #[serde(
        default,
        deserialize_with = "count_or_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub previous_total: Option<String>,
    #[serde(default)]
    pub data: Vec<MeasurePoint>,
}

/// One day of a measure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasurePoint {
    #[serde(default)]
    pub date: String,
    /// A **string** here, like every other count Mastodon sends: the dashboard answers
    /// `{"date":"2026-11-01","value":"0"}`, and a `u64` would reject that payload with
    /// `invalid type: string "0", expected u64`. This instance's own API sends the same
    /// count as a JSON number, so both spellings are accepted.
    #[serde(default, deserialize_with = "count")]
    pub value: String,
}

/// A domain whose federation this instance allows even when it blocks federation
/// generally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainAllow {
    pub id: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub created_at: String,
}

/// Counts that arrive either as a JSON number or as a string.
///
/// Mastodon writes every count as a string, this instance writes numbers, and a dashboard
/// should not have to know which server it is reading.
fn count_or_null<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    /// Either spelling of one count.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Incoming {
        Text(String),
        Number(i64),
    }

    Ok(
        Option::<Incoming>::deserialize(deserializer)?.map(|value| match value {
            Incoming::Text(text) => text,
            Incoming::Number(number) => number.to_string(),
        }),
    )
}

/// One count as a string, whatever spelling it arrived in.
fn count<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(count_or_null(deserializer)?.unwrap_or_default())
}

/// Which reports to put in one page of the queue.
///
/// Every field is optional and every absent one means "do not narrow": a moderator who
/// has chosen nothing should see the whole queue, not an empty one.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportFilter {
    /// `None` asks for every state, which is what an untouched queue shows.
    #[serde(default)]
    pub state: Option<ReportState>,
    #[serde(default)]
    pub category: Option<String>,
    /// The account being reported.
    #[serde(default)]
    pub target_account_id: Option<String>,
    /// The account that filed the report.
    #[serde(default)]
    pub account_id: Option<String>,
    /// The moderator who claimed it.
    #[serde(default)]
    pub assigned_account_id: Option<String>,
    /// Only reports nobody has claimed.
    #[serde(default)]
    pub unassigned: bool,
    #[serde(default)]
    pub limit: Option<u16>,
    #[serde(default)]
    pub offset: Option<u32>,
}

/// Where an account lives, for the directory filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountOrigin {
    /// An account this instance hosts.
    Local,
    /// An account it has only seen over federation.
    Remote,
}

impl AccountOrigin {
    /// The query value the API expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
        }
    }
}

/// The standing a moderator is filtering the directory by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    /// Approved, neither silenced nor suspended.
    Active,
    /// Waiting for a decision.
    Pending,
    /// Never approved, so it cannot sign in.
    Disabled,
    /// Limited to its followers.
    Silenced,
    /// Refused entirely.
    Suspended,
    /// No narrowing, which is not the same as [`Active`](Self::Active).
    All,
}

impl AccountStatus {
    /// The query value the API expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Pending => "pending",
            Self::Disabled => "disabled",
            Self::Silenced => "silenced",
            Self::Suspended => "suspended",
            Self::All => "all",
        }
    }
}

/// What the account directory is being asked for.
///
/// Searches are case-insensitive substring matches on the server, so `username` is a
/// fragment rather than an exact name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminAccountQuery {
    #[serde(default)]
    pub origin: Option<AccountOrigin>,
    #[serde(default)]
    pub status: Option<AccountStatus>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    /// Matches the address stored for local accounts.
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub by_domain: Option<String>,
    /// Only administrators.
    #[serde(default)]
    pub staff: bool,
    #[serde(default)]
    pub limit: Option<u16>,
    #[serde(default)]
    pub offset: Option<u32>,
}

/// A moderator's private note on an account.
///
/// Notes are never federated and never shown to the account; they are the memory of why
/// a decision was taken.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountNote {
    pub id: String,
    pub account_id: String,
    /// Who wrote it.
    #[serde(default)]
    pub author_id: String,
    #[serde(default)]
    pub author_username: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub created_at: String,
}

/// Everything a moderator needs about one account, in one response.
///
/// The account's own fields arrive at the top level of the payload, so they are flattened
/// here rather than nested under a key the server does not send.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountDetail {
    /// The directory row: standing, counts, and the public account.
    #[serde(flatten)]
    pub account: AdminAccount,
    #[serde(default)]
    pub notes: Vec<AccountNote>,
    /// What a scanner has said about this account's content, newest first. The detail view
    /// sends a trimmed flag, so `scan_id` and `account_id` may be empty here.
    #[serde(default)]
    pub flags: Vec<ScanFlag>,
    /// How many reports name this account, resolved ones included.
    #[serde(default)]
    pub reports_against: i64,
}

/// What a moderator decided about a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlagState {
    /// Nobody has looked at it yet.
    #[default]
    Open,
    /// Looked at and judged harmless.
    Dismissed,
    /// Looked at and acted on.
    Actioned,
}

impl FlagState {
    /// The state name the API expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Dismissed => "dismissed",
            Self::Actioned => "actioned",
        }
    }
}

/// One scan result a moderator has to judge: what a scanner found, and how strongly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanFlag {
    pub id: String,
    /// The scan that produced it; empty on the trimmed flag inside an account detail.
    #[serde(default)]
    pub scan_id: String,
    #[serde(default)]
    pub status_id: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    /// Why it was flagged, in the scanner's own words.
    #[serde(default)]
    pub reason: String,
    /// The scanner's confidence in its strongest label.
    #[serde(default)]
    pub score: Option<f32>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub severity: ScanSeverity,
    #[serde(default)]
    pub state: FlagState,
    #[serde(default)]
    pub created_at: String,
}

/// What a label, once confident enough, means for the content carrying it.
///
/// The scanner reports labels; the administrator decides what each label is worth. That
/// is why this enum lives here and not in the scanner: it is a policy, not a detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanSeverity {
    /// Record it and say nothing.
    #[default]
    Ignore,
    /// Put it in the review queue.
    Flag,
    /// Treat the subject as sensitive without refusing it.
    Sensitive,
    /// Refuse the content outright.
    Block,
}

impl ScanSeverity {
    /// The severity name the API expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ignore => "ignore",
            Self::Flag => "flag",
            Self::Sensitive => "sensitive",
            Self::Block => "block",
        }
    }
}

/// The kind of content a scanner or a scan is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScannerKind {
    Image,
    Video,
    Text,
}

impl ScannerKind {
    /// The kind name the API expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
            Self::Text => "text",
        }
    }
}

impl Default for ScannerKind {
    /// Used only when a payload omits the kind; image is the first kind the server knows.
    fn default() -> Self {
        Self::Image
    }
}

/// One decision about one label: this label, this confident, means this much.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScannerRule {
    pub id: String,
    pub label: String,
    /// Probability between 0 and 1 below which the label is ignored.
    #[serde(default)]
    pub threshold: f64,
    #[serde(default)]
    pub severity: ScanSeverity,
}

/// A scanner service this instance controls.
///
/// A service is headless: it answers a manifest when asked, reports labels, and is told
/// what those labels mean. `labels` and `manifest` stay empty until a refresh succeeds,
/// which is the difference between "configured" and "reachable".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scanner {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: ScannerKind,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub enabled: bool,
    /// The content set the service verifies its copy against.
    #[serde(default)]
    pub asset_id: Option<String>,
    /// Manifest version the service speaks.
    #[serde(default)]
    pub contract: i32,
    #[serde(default)]
    pub labels: Vec<String>,
    /// The manifest as the service sent it, kept verbatim for the console.
    #[serde(default)]
    pub manifest: serde_json::Value,
    #[serde(default)]
    pub manifest_at: Option<String>,
    #[serde(default)]
    pub timeout_ms: i32,
    #[serde(default)]
    pub rules: Vec<ScannerRule>,
}

/// The scanner fields an administrator sets.
///
/// There is no `id` and no manifest: those come back from the server, and a create that
/// carried them would be claiming a service had answered before it was ever asked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewScanner {
    pub name: String,
    pub kind: ScannerKind,
    /// The service's base URL, with scheme.
    pub endpoint: String,
    /// Shared secret the service proves itself with. A scanner without one cannot be
    /// configured remotely, because it has nothing to prove itself with.
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default = "scanner_enabled_by_default")]
    pub enabled: bool,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default = "scanner_default_timeout_ms")]
    pub timeout_ms: i32,
}

impl NewScanner {
    /// A scanner that is enabled, has no content set, and times out at five seconds.
    #[must_use]
    pub fn new(name: impl Into<String>, kind: ScannerKind, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind,
            endpoint: endpoint.into(),
            secret: None,
            enabled: true,
            asset_id: None,
            timeout_ms: scanner_default_timeout_ms(),
        }
    }
}

const fn scanner_enabled_by_default() -> bool {
    true
}

/// Matches the server's default, so a caller that does not care gets the same number.
const fn scanner_default_timeout_ms() -> i32 {
    5000
}

/// What a content set holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    /// Known-bad content digests.
    Hashes,
    /// Model weights.
    Weights,
    /// Word and phrase lists.
    Words,
}

impl AssetKind {
    /// The kind name the API expects.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hashes => "hashes",
            Self::Weights => "weights",
            Self::Words => "words",
        }
    }
}

impl Default for AssetKind {
    /// Used only when a payload omits the kind; a digest list is the commonest set.
    fn default() -> Self {
        Self::Hashes
    }
}

/// One content set a scanner verifies its copy of the data against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerAsset {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub kind: AssetKind,
    /// SHA-256 of the bytes, computed by the server rather than accepted from a client.
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub created_at: String,
}

/// A content set on its way up.
///
/// The bytes travel base64-encoded in the body so a browser on a machine that never sees
/// the server's filesystem can still install one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetUpload {
    pub name: String,
    pub kind: AssetKind,
    #[serde(default = "octet_stream")]
    pub content_type: String,
    /// The bytes, standard base64.
    pub data: String,
}

fn octet_stream() -> String {
    "application/octet-stream".to_owned()
}

impl AssetUpload {
    /// An upload with the generic content type.
    #[must_use]
    pub fn new(name: impl Into<String>, kind: AssetKind, data: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind,
            content_type: octet_stream(),
            data: data.into(),
        }
    }
}

/// A subject queued for scanning by hand.
///
/// The sweep queues new content on its own; this is for re-running something a moderator
/// is looking at now, which is why the subject is named explicitly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanRequest {
    /// `status` or `media`.
    pub subject_type: String,
    pub subject_id: String,
    pub kind: ScannerKind,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub mime: Option<String>,
}

/// One scan: a subject given to one scanner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scan {
    pub id: String,
    #[serde(default)]
    pub scanner_id: String,
    #[serde(default)]
    pub kind: ScannerKind,
    /// `status` or `media`.
    #[serde(default)]
    pub subject_type: String,
    #[serde(default)]
    pub subject_id: String,
    /// `pending`, `running`, `done` or `failed`; kept as a string for the same reason as
    /// [`Report::state`].
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub score: Option<f32>,
    #[serde(default)]
    pub labels: Vec<String>,
    /// Why the scanner could not answer, when it could not.
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub finished_at: Option<String>,
}

/// The four numbers the console shows before anything else.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminCounts {
    #[serde(default)]
    pub open_reports: i64,
    #[serde(default)]
    pub unassigned_reports: i64,
    #[serde(default)]
    pub pending_accounts: i64,
    #[serde(default)]
    pub open_flags: i64,
}

/// What the scanning pipeline is doing, in five numbers.
///
/// `queued` growing without bound is the sign of a scanner that is down, and a moderator
/// should be told that rather than have to infer it from an empty queue.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanningSummary {
    #[serde(default)]
    pub scanners: i64,
    #[serde(default)]
    pub enabled: i64,
    #[serde(default)]
    pub rules: i64,
    #[serde(default)]
    pub assets: i64,
    #[serde(default)]
    pub queued: i64,
}

/// How many open flags carry one severity, so the console can explain a number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeverityCount {
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub count: i64,
}

/// What a grant belongs to, what it may do, and how much work is waiting.
///
/// Fetched once when the console opens: a client that had to assemble this from four
/// requests would show a half-built console whenever one of them failed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminMeta {
    /// The account behind the token.
    pub account: Account,
    /// Whether that account is an administrator, not merely a moderator.
    #[serde(default)]
    pub staff: bool,
    /// Scopes the token actually carries.
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub counts: AdminCounts,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub scanning: ScanningSummary,
    /// Open flags per severity.
    #[serde(default)]
    pub severities: Vec<SeverityCount>,
}

/// The flat counts behind the overview screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdminOverview {
    #[serde(default)]
    pub accounts: i64,
    #[serde(default)]
    pub local_accounts: i64,
    #[serde(default)]
    pub active_accounts: i64,
    #[serde(default)]
    pub statuses: i64,
    #[serde(default)]
    pub open_reports: i64,
    #[serde(default)]
    pub resolved_reports: i64,
    #[serde(default)]
    pub unassigned_reports: i64,
    #[serde(default)]
    pub pending_accounts: i64,
    #[serde(default)]
    pub silences: i64,
    #[serde(default)]
    pub suspensions: i64,
    #[serde(default)]
    pub domain_blocks: i64,
    #[serde(default)]
    pub email_domain_blocks: i64,
    #[serde(default)]
    pub staff: i64,
}

/// A mail domain registrations are refused from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailDomainBlock {
    pub id: String,
    #[serde(default)]
    pub domain: String,
    /// Whether the block is advisory only, leaving existing accounts alone.
    #[serde(default)]
    pub allow_registration: bool,
    #[serde(default)]
    pub created_at: String,
}

/// What a bulk block did.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainImportOutcome {
    /// How many domains were blocked.
    #[serde(default)]
    pub blocked: u64,
    /// The entries that were not a domain at all, so the caller can show them back rather
    /// than silently lose half a list.
    #[serde(default)]
    pub skipped: Vec<String>,
}

/// A block to create or replace.
///
/// The server requires the whole block on an update, not a patch, so one type serves both
/// writes: a partial update would clear whichever fields it left out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainBlockDraft {
    pub domain: String,
    #[serde(default)]
    pub severity: DomainSeverity,
    #[serde(default)]
    pub reject_media: bool,
    #[serde(default)]
    pub reject_reports: bool,
    #[serde(default)]
    pub obfuscate: bool,
    #[serde(default)]
    pub public_comment: String,
}

impl DomainBlockDraft {
    /// A block at the given severity, with nothing else set.
    #[must_use]
    pub fn new(domain: impl Into<String>, severity: DomainSeverity) -> Self {
        Self {
            domain: domain.into(),
            severity,
            reject_media: false,
            reject_reports: false,
            obfuscate: false,
            public_comment: String::new(),
        }
    }
}

/// A report a person is filing.
///
/// Filing is not an administrative act: any signed-in account can do it, and the server
/// records who filed it, not who the caller claims to be.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportDraft {
    /// The account being reported.
    pub account_id: String,
    #[serde(default)]
    pub status_ids: Vec<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub rule_ids: Vec<String>,
}

impl ReportDraft {
    /// A report of one account, with no posts quoted yet.
    #[must_use]
    pub fn new(account_id: impl Into<String>) -> Self {
        Self {
            account_id: account_id.into(),
            status_ids: Vec::new(),
            category: String::new(),
            comment: String::new(),
            rule_ids: Vec::new(),
        }
    }
}

/// A warning a moderator can reuse instead of retyping it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarningPreset {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub created_at: String,
}

/// Something the instance noticed that an administrator should see.
///
/// An alert is a record, not a page: `detail` holds whatever the reporter found useful,
/// and acknowledging one changes exactly one field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminAlert {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub status: Option<i32>,
    #[serde(default)]
    pub remote_ip: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub detail: serde_json::Value,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub acknowledged_at: Option<String>,
    #[serde(default)]
    pub acknowledged_by: Option<String>,
}
