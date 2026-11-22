//! The capability answer for servers that publish no capability list.
//!
//! Nodus answers "what do you serve?" with `GET /api/v1/meta`. Mastodon and Misskey
//! publish no such document, so the only honest answer is a table built from what the
//! running servers actually answer. The rule is the status code's meaning for this
//! route: **404 = the route is absent**, **401/403 = the route exists and refused this
//! caller**, **200/204/400/422 = the route exists** (a `400` is a rejected parameter
//! list, which still proves the handler is wired). A row with no route at all was
//! confirmed against the server's own route table — Mastodon `rails routes`, Misskey
//! `POST /api/endpoints` — because probing one invented path proves only that path is
//! absent, not that the concept is missing.
//!
//! Two observations are deliberately not treated as evidence. A Mastodon `500` in this
//! lab is the frontend asset manifest failing while Rails renders its error page
//! (`ViteRuby::MissingEntrypointError`), so it says nothing about the route; and
//! `GET /api/v1/instance/domain_blocks` answers `404` when the instance setting disables
//! that public API (`require_enabled_api!`), which is a configured-off surface.
//!
//! Probed 2026-11-22 against Mastodon `v4.6.3` (`127.0.0.1:18081`) and Misskey
//! `2026.11.0` (`127.0.0.1:18082`) in the local lab.
//!
//! ## Admin-only surfaces count as served
//!
//! A `403` is a present capability: the server has the route, the signed-in user is not
//! an admin. Those rows are `true` — `admin`, `moderation`, `remote_blocks` and
//! `remote_accounts` on both protocols, plus Misskey's `report_forwarding`. The baseline
//! answers one question, "does this server serve that surface?", and the route alone
//! answers it; whether the current user may call it is a property of the session, which
//! this answer does not carry. The frontend therefore gates admin-only screens on the
//! signed-in user's role and uses the capability only to ask whether the surface exists.
//! Claiming `false` instead was rejected because it hides a working screen from the one
//! user who could use it — the same class of error as drawing a control that 404s.
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
//!
//! Everything Nodus adds beyond a protocol's own client API stays `false`.

use crate::feature::Feature;

/// The Mastodon baseline, probed against `v4.6.3` on 2026-11-22; see the module table
/// for the status code behind each row.
const MASTODON: &[(Feature, bool)] = &[
    (Feature::Admin, true),
    (Feature::Moderation, true),
    (Feature::Scanners, false),
    (Feature::Topology, false),
    (Feature::Protocol, false),
    (Feature::RemoteBlocks, true),
    (Feature::Polls, true),
    (Feature::Batch, false),
    (Feature::Appeals, false),
    (Feature::InstanceRules, true),
    (Feature::ModerationRules, false),
    (Feature::MediaQuarantine, false),
    (Feature::BlockLists, false),
    (Feature::ModerationStats, false),
    (Feature::RemoteAccounts, true),
    (Feature::Warnings, false),
    (Feature::Escalation, false),
    (Feature::FederationDelivery, false),
    (Feature::DomainPauses, false),
    (Feature::SettingsTransfer, false),
    (Feature::ReportForwarding, false),
    (Feature::AccountDeletion, false),
    (Feature::Drive, false),
    (Feature::DriveShare, false),
    (Feature::DriveAttach, false),
    (Feature::FeedAccounts, false),
    (Feature::Events, false),
    (Feature::SyncToken, false),
    (Feature::Favourites, true),
];

/// The Misskey baseline, probed against `2026.11.0` on 2026-11-22; see the module table
/// for the status code behind each row.
const MISSKEY: &[(Feature, bool)] = &[
    (Feature::Admin, true),
    (Feature::Moderation, true),
    (Feature::Scanners, false),
    (Feature::Topology, false),
    (Feature::Protocol, false),
    (Feature::RemoteBlocks, true),
    (Feature::Polls, true),
    (Feature::Batch, false),
    (Feature::Appeals, false),
    (Feature::InstanceRules, true),
    (Feature::ModerationRules, false),
    (Feature::MediaQuarantine, false),
    (Feature::BlockLists, false),
    (Feature::ModerationStats, false),
    (Feature::RemoteAccounts, true),
    (Feature::Warnings, false),
    (Feature::Escalation, false),
    (Feature::FederationDelivery, false),
    (Feature::DomainPauses, false),
    (Feature::SettingsTransfer, false),
    (Feature::ReportForwarding, true),
    (Feature::AccountDeletion, true),
    (Feature::Drive, true),
    (Feature::DriveShare, true),
    (Feature::DriveAttach, true),
    (Feature::FeedAccounts, false),
    (Feature::Events, false),
    (Feature::SyncToken, false),
    (Feature::Favourites, true),
];

/// Whether a Mastodon instance serves `feature` according to the baseline table.
pub(crate) fn mastodon_supports(feature: Feature) -> bool {
    MASTODON
        .iter()
        .find(|(named, _)| *named == feature)
        .is_some_and(|(_, served)| *served)
}

/// Whether a Misskey instance serves `feature` according to the baseline table.
pub(crate) fn misskey_supports(feature: Feature) -> bool {
    MISSKEY
        .iter()
        .find(|(named, _)| *named == feature)
        .is_some_and(|(_, served)| *served)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every row as probed, so a future edit to a table cannot flip one silently.
    const EXPECTED: &[(Feature, bool, bool)] = &[
        (Feature::Admin, true, true),
        (Feature::Moderation, true, true),
        (Feature::Scanners, false, false),
        (Feature::Topology, false, false),
        (Feature::Protocol, false, false),
        (Feature::RemoteBlocks, true, true),
        (Feature::Polls, true, true),
        (Feature::Batch, false, false),
        (Feature::Appeals, false, false),
        (Feature::InstanceRules, true, true),
        (Feature::ModerationRules, false, false),
        (Feature::MediaQuarantine, false, false),
        (Feature::BlockLists, false, false),
        (Feature::ModerationStats, false, false),
        (Feature::RemoteAccounts, true, true),
        (Feature::Warnings, false, false),
        (Feature::Escalation, false, false),
        (Feature::FederationDelivery, false, false),
        (Feature::DomainPauses, false, false),
        (Feature::SettingsTransfer, false, false),
        (Feature::ReportForwarding, false, true),
        (Feature::AccountDeletion, false, true),
        (Feature::Drive, false, true),
        (Feature::DriveShare, false, true),
        (Feature::DriveAttach, false, true),
        (Feature::FeedAccounts, false, false),
        (Feature::Events, false, false),
        (Feature::SyncToken, false, false),
        (Feature::Favourites, true, true),
    ];

    #[test]
    fn the_matrix_pins_every_row_as_probed() {
        assert_eq!(EXPECTED.len(), Feature::ALL.len());
        for &(feature, mastodon, misskey) in EXPECTED {
            assert_eq!(mastodon_supports(feature), mastodon, "mastodon {feature:?}");
            assert_eq!(misskey_supports(feature), misskey, "misskey {feature:?}");
        }
    }

    #[test]
    fn each_table_names_every_feature_exactly_once() {
        for table in [MASTODON, MISSKEY] {
            assert_eq!(table.len(), Feature::ALL.len());
            for feature in Feature::ALL {
                let count = table.iter().filter(|(named, _)| *named == feature).count();
                assert_eq!(count, 1, "{feature:?} appears {count} times");
            }
        }
    }
}
