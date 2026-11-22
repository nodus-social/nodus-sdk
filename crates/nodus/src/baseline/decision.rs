//! The per-feature decision and the evidence behind it.
//!
//! One row per [`Feature`], in `Feature::ALL` order. Each row records the probe that
//! settled it for Mastodon 4.6.3 and Misskey 2026.11.0; a row with no route was checked
//! against the server's route table (Mastodon `rails routes`, Misskey
//! `POST /api/endpoints`), because probing one invented path proves only that path is
//! absent.
//!
//! | Feature | Mastodon 4.6.3 | Misskey 2026.11.0 |
//! |---|---|---|
//! | `admin` | yes — `403 GET /api/v1/admin/reports` | yes — `200 POST /api/admin/meta` |
//! | `moderation` | yes — `403 GET /api/v1/admin/reports` | yes — `200 POST /api/admin/abuse-user-reports` |
//! | `scanners` | no concept, no route | no concept, no route |
//! | `topology` | no — `instance/peers` is a peer list, not a graph | no — `federation/instances` is a list |
//! | `protocol` | no — no route for `GET /api/v1/meta` | no — `POST /api/meta` has no feature list |
//! | `remote_blocks` | yes — `403 GET /api/v1/admin/domain_blocks` | yes — `400 POST /api/admin/federation/update-instance` |
//! | `polls` | yes — `400 POST /api/v1/polls/:id/votes` | yes — `400 POST /api/notes/polls/vote` |
//! | `batch` | no concept, no route | no concept, no route |
//! | `appeals` | no — `404 GET /api/v1/appeals`, only a web flow | no concept, no route |
//! | `instance_rules` | yes — `200 GET /api/v1/instance/rules` | yes — `200 POST /api/meta` `serverRules` |
//! | `moderation_rules` | no — `admin/rules` is the instance rules | no concept, no route |
//! | `media_quarantine` | no concept, no route | no concept, no route |
//! | `block_lists` | no — `/api/v1/blocks` is the caller's own list | no — `i/export-blocking` is the own list |
//! | `moderation_stats` | no — `admin/measures` is instance statistics | no — `admin/get-index-stats` is instance statistics |
//! | `remote_accounts` | yes — `403 GET /api/v1/admin/accounts` | yes — `200 POST /api/admin/show-users` |
//! | `warnings` | no — `404 GET /api/v1/admin/warnings` | no concept, no route |
//! | `escalation` | no concept, no route | no concept, no route |
//! | `federation_delivery` | no concept, no route | no concept, no route |
//! | `domain_pauses` | no concept, no route | no — `admin/queue/pause` pauses the queue |
//! | `settings_transfer` | no concept, no route | no — `i/registry/get-all` only stores settings |
//! | `report_forwarding` | no — forwarding is automatic, no route | yes — `400 POST /api/admin/forward-abuse-user-report` |
//! | `account_deletion` | no client route | yes — `400 POST /api/i/delete-account` |
//! | `drive` | no concept, no route | yes — `400 POST /api/drive/files/create` |
//! | `drive_share` | no concept, no route | yes — `drive/files/create` with `isLink` |
//! | `drive_attach` | no concept, no route | yes — `400 POST /api/notes/create` `fileIds` |
//! | `feed_accounts` | no concept, no route | no concept, no route |
//! | `events` | no concept, no route | no concept, no route |
//! | `sync_token` | no concept, no route | no concept, no route |
//! | `favourites` | yes — `200 GET /api/v1/favourites` | yes — `200 POST /api/i/favorites` |
//! | `bookmarks` | yes — `200 GET /api/v1/bookmarks` | no — `404 POST /api/i/bookmarks`; `i/favorites` is `favourites` |
//! | `conversations` | yes — `200 GET /api/v1/conversations` | no — `404 POST /api/conversations`; `chat/history` is chat |
//! | `lists` | yes — `200 GET /api/v1/lists` | yes — `200 POST /api/users/lists/list` |
//! | `filters` | yes — `200 GET /api/v2/filters` (and `/api/v1/filters`) | no — `404 POST /api/filters/list`; `mute/list` is user mutes |
//! | `scheduled_statuses` | yes — `200 GET /api/v1/scheduled_statuses` | no — `404 POST /api/scheduled-statuses`; `notes/drafts/list` is a draft |
//! | `notifications` | yes — `200 GET /api/v1/notifications` | yes — `200 POST /api/i/notifications` |
//! | `announcements` | yes — `200 GET /api/v1/announcements` | yes — `200 POST /api/announcements` |
//! | `search` | yes — `200 GET /api/v2/search` | yes — `400 POST /api/notes/search` (rejected param, route present) |
//! | `directory` | yes — `200 GET /api/v1/directory` | yes — `200 POST /api/users` (local user directory) |
//! | `markers` | yes — `200 GET /api/v1/markers` | no — `404 POST /api/i/markers` |
//! | `preferences` | yes — `200 GET /api/v1/preferences` | no — `404 POST /api/i/preferences`; `i/registry/get` is client storage |
//! | `streaming` | yes — `301 GET /api/v1/streaming` to the canonical host; the streaming process ships with Mastodon but is not started in this lab | yes — `503 GET /streaming` without a websocket upgrade |
//! | `tags` | yes — `200 GET /api/v1/tags/test`, follow/unfollow `200` | no — `404 POST /api/hashtags/follow`; `hashtags/show` is a tag page only |

use crate::feature::Feature;

/// One feature decided for all three protocols, in `Feature::ALL` order.
///
/// The exhaustive `match` is the compile-time gate: a new `Feature` variant is a
/// non-exhaustive-match error here, so it cannot reach a caller without a decision for
/// Nodus, Mastodon and Misskey. The three bools are `[nodus, mastodon, misskey]`.
///
/// Nodus is `false` in every row on purpose: it publishes `/api/v1/meta`, so its answer
/// is the published list, never a baseline guess. The flag exists so the compiler still
/// asks about Nodus when a feature is added, even though the runtime answer ignores it.
#[allow(clippy::match_same_arms)] // one arm per Feature keeps each decision visible
pub(super) const fn decision(feature: Feature) -> [bool; 3] {
    match feature {
        Feature::Admin => [false, true, true],
        Feature::Moderation => [false, true, true],
        Feature::Scanners => [false, false, false],
        Feature::Topology => [false, false, false],
        Feature::Protocol => [false, false, false],
        Feature::RemoteBlocks => [false, true, true],
        Feature::Polls => [false, true, true],
        Feature::Batch => [false, false, false],
        Feature::Appeals => [false, false, false],
        Feature::InstanceRules => [false, true, true],
        Feature::ModerationRules => [false, false, false],
        Feature::MediaQuarantine => [false, false, false],
        Feature::BlockLists => [false, false, false],
        Feature::ModerationStats => [false, false, false],
        Feature::RemoteAccounts => [false, true, true],
        Feature::Warnings => [false, false, false],
        Feature::Escalation => [false, false, false],
        Feature::FederationDelivery => [false, false, false],
        Feature::DomainPauses => [false, false, false],
        Feature::SettingsTransfer => [false, false, false],
        Feature::ReportForwarding => [false, false, true],
        Feature::AccountDeletion => [false, false, true],
        Feature::Drive => [false, false, true],
        Feature::DriveShare => [false, false, true],
        Feature::DriveAttach => [false, false, true],
        Feature::FeedAccounts => [false, false, false],
        Feature::Events => [false, false, false],
        Feature::SyncToken => [false, false, false],
        Feature::Favourites => [false, true, true],
        Feature::Bookmarks => [false, true, false],
        Feature::Conversations => [false, true, false],
        Feature::Lists => [false, true, true],
        Feature::Filters => [false, true, false],
        Feature::ScheduledStatuses => [false, true, false],
        Feature::Notifications => [false, true, true],
        Feature::Announcements => [false, true, true],
        Feature::Search => [false, true, true],
        Feature::Directory => [false, true, true],
        Feature::Markers => [false, true, false],
        Feature::Preferences => [false, true, false],
        Feature::Streaming => [false, true, true],
        Feature::Tags => [false, true, false],
    }
}
