//! The per-feature decision and the evidence behind it.
//!
//! One row per [`Feature`], in `Feature::ALL` order. Each row records the probe that
//! settled it for Mastodon 4.6.3 and Misskey 2026.11.0; a row with no route was checked
//! against the server's route table (Mastodon `rails routes`, Misskey
//! `POST /api/endpoints`), because probing one invented path proves only that path is
//! absent.
//!
//! `moderation_rules` is not a new spelling: it is the existing `Feature::ModerationRules`
//! row above, whose bit is unchanged, and it is the moderation-patterns section of the
//! vocabulary. The published list therefore names 70 capabilities, not 71.
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
//! | `admin_overview` | yes — DECLARED: `/admin/dashboard` is in `rails routes`, but the page has no API of its own | yes — `200 POST /api/admin/meta` |
//! | `admin_announcements` | yes — DECLARED: `/admin/announcements` is in `rails routes`; the public `/api/v1/announcements` is a different surface | yes — `200 POST /api/admin/announcements/list` |
//! | `admin_audit` | yes — DECLARED: `/admin/action_logs` is in `rails routes`; `GET /api/v1/admin/audit` is not routed | yes — `200 POST /api/admin/show-moderation-logs` (Misskey's admin action log) |
//! | `admin_domains` | yes — `403 GET /api/v1/admin/domain_allows` | no — no allow-list; instance control is `admin_federation` |
//! | `admin_email` | yes — `403 GET /api/v1/admin/email_domain_blocks` | no concept, no route |
//! | `admin_escalation` | no concept, no route | no concept, no route |
//! | `admin_federation` | no — no client route for the delivery queue | yes — `200 POST /api/admin/queue/queues` (delivery queue page) |
//! | `admin_protocol` | no — the page is client-side detection, the server contributes no route | no — same client-side detection |
//! | `admin_replies` | yes — DECLARED: `/admin/warning_presets` is in `rails routes`; `GET /api/v1/admin/warning_presets` is not routed | no concept, no route |
//! | `admin_rules` | yes — DECLARED: `/admin/rules` is in `rails routes`, but the page has no API of its own | yes — DECLARED: rules are edited through `POST /api/admin/update-meta` `serverRules`, with no separate route |
//! | `admin_scanners` | no concept, no route | no concept, no route |
//! | `admin_settings` | yes — DECLARED: `/admin/settings/{branding,registrations,about,…}` is in `rails routes`; the settings have no client API | yes — `200 POST /api/admin/meta` (settings live in the single admin meta document) |
//! | `admin_topology` | no concept, no route | no — `admin/server-info` is host health, not a process graph |
//! | `moderation_accounts` | yes — `403 GET /api/v1/admin/accounts` | yes — `200 POST /api/admin/show-users` |
//! | `moderation_appeals` | yes — DECLARED: `/admin/disputes/appeals` is in `rails routes`, but the page has no client API | no concept, no route |
//! | `moderation_federation` | yes — `403 GET /api/v1/admin/domain_blocks` | yes — `400 POST /api/admin/federation/update-instance` (instance block/silence) |
//! | `moderation_flags` | no — a scanner-flag queue needs `admin_scanners`, which is absent | no concept, no route |
//! | `moderation_media` | no — no quarantine queue | no — `admin/drive/files` is storage management, not quarantine |
//! | `moderation_registrations` | yes — `403 GET /api/v2/admin/accounts` (pending registrations) | yes — `200 POST /api/admin/invite/list` |
//! | `moderation_reports` | yes — `403 GET /api/v1/admin/reports` | yes — `200 POST /api/admin/abuse-user-reports` |
//! | `mutes` | yes — `401 GET /api/v1/mutes` (user-scoped, `routes/api.rb`) | yes — `401 POST /api/mute/list` |
//! | `blocks` | yes — `401 GET /api/v1/blocks` (user-scoped, `routes/api.rb`) | yes — `401 POST /api/blocking/list` |
//! | `domain_blocks` | yes — `401 GET /api/v1/domain_blocks`, the account's own list; `/api/v1/instance/domain_blocks` is the public instance list, not this | yes — `400 POST /api/i/update` `mutedInstances`: muting an instance is the personal list (settings → mute-block → instance-mute) |
//! | `sessions` | yes — DECLARED: `/auth/edit` (`registrations#edit`) renders the session list (`views/auth/registrations/_sessions.html.haml`) and `DELETE /settings/sessions/:id` is in `routes/settings.rb`; there is no client API | no — `404 POST /api/i/sessions`; `i/signin-history` is a read-only login log and `i/revoke-token` revokes an app token, neither lists nor revokes a session |
//! | `two_factor` | yes — DECLARED: `/settings/two_factor_authentication_methods` is in `routes/settings.rb` (302 to sign-in), as are the `two_factor_authentication` enrolment routes; no client API | yes — `400 POST /api/i/2fa/register` (plus `i/2fa/done`, `-remove-key`, `-register-key`) |
//! | `exports` | yes — DECLARED: `/settings/export` and the CSV `/settings/exports/{follows,blocks,mutes,lists,domain_blocks,bookmarks,custom_filters}` routes are in `routes/settings.rb` (302 to sign-in); no client API | yes — `400 POST /api/i/export-notes`, plus `i/export-following`, `-blocking`, `-mute`, `-favorites`, `-user-lists`, `-antennas`, `-clips` |
//! | `featured_tags` | yes — `404 GET /api/v1/accounts/1/featured_tags` with `{"error":"Record not found"}`, so the route ran and only the account was absent; Nodus publishes the name for `GET|POST /api/v1/featured_tags` and `/suggestions` | no — `POST /api/endpoints` lists no featured-tag route; profile tags are not a Misskey object |
//! | `notification_settings` | yes — `401 GET /api/v1/notifications/policy`, the route is in `routes/api.rb` and only auth is missing | no — `POST /api/endpoints` lists no notification-policy route; Misskey keeps client notification settings in `i/registry`, not on the server |

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
        Feature::AdminOverview => [false, true, true],
        Feature::AdminAnnouncements => [false, true, true],
        Feature::AdminAudit => [false, true, true],
        Feature::AdminDomains => [false, true, false],
        Feature::AdminEmail => [false, true, false],
        Feature::AdminEscalation => [false, false, false],
        Feature::AdminFederation => [false, false, true],
        Feature::AdminProtocol => [false, false, false],
        Feature::AdminReplies => [false, true, false],
        Feature::AdminRules => [false, true, true],
        Feature::AdminScanners => [false, false, false],
        Feature::AdminSettings => [false, true, true],
        Feature::AdminTopology => [false, false, false],
        Feature::ModerationAccounts => [false, true, true],
        Feature::ModerationAppeals => [false, true, false],
        Feature::ModerationFederation => [false, true, true],
        Feature::ModerationFlags => [false, false, false],
        Feature::ModerationMedia => [false, false, false],
        Feature::ModerationRegistrations => [false, true, true],
        Feature::ModerationReports => [false, true, true],
        Feature::Mutes => [false, true, true],
        Feature::Blocks => [false, true, true],
        Feature::DomainBlocks => [false, true, true],
        Feature::Sessions => [false, true, false],
        Feature::TwoFactor => [false, true, true],
        Feature::Exports => [false, true, true],
        Feature::FeaturedTags => [false, true, false],
        Feature::NotificationSettings => [false, true, false],
    }
}
