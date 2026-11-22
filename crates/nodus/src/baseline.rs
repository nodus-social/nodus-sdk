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

use crate::active::ActiveFeatures;
use crate::capabilities::Provider;
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
const fn decision(feature: Feature) -> [bool; 3] {
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
    }
}

/// The active set a provider serves when it publishes no capability list.
///
/// Nodus returns an empty set: without a published list there is nothing honest to
/// claim, and [`crate::Capabilities::from_published`] supplies that list.
pub(crate) const fn active(provider: Provider) -> ActiveFeatures {
    let column = match provider {
        Provider::Nodus => 0,
        Provider::Mastodon => 1,
        Provider::Misskey => 2,
    };
    let mut set = ActiveFeatures::NONE;
    let mut index = 0;
    while index < Feature::COUNT {
        let feature = Feature::ALL[index];
        if decision(feature)[column] {
            set.insert(feature);
        }
        index += 1;
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every row as probed: `(feature, nodus, mastodon, misskey)`. Nodus is always
    /// false because it has no baseline; see [`decision`].
    const EXPECTED: &[(Feature, bool, bool, bool)] = &[
        (Feature::Admin, false, true, true),
        (Feature::Moderation, false, true, true),
        (Feature::Scanners, false, false, false),
        (Feature::Topology, false, false, false),
        (Feature::Protocol, false, false, false),
        (Feature::RemoteBlocks, false, true, true),
        (Feature::Polls, false, true, true),
        (Feature::Batch, false, false, false),
        (Feature::Appeals, false, false, false),
        (Feature::InstanceRules, false, true, true),
        (Feature::ModerationRules, false, false, false),
        (Feature::MediaQuarantine, false, false, false),
        (Feature::BlockLists, false, false, false),
        (Feature::ModerationStats, false, false, false),
        (Feature::RemoteAccounts, false, true, true),
        (Feature::Warnings, false, false, false),
        (Feature::Escalation, false, false, false),
        (Feature::FederationDelivery, false, false, false),
        (Feature::DomainPauses, false, false, false),
        (Feature::SettingsTransfer, false, false, false),
        (Feature::ReportForwarding, false, false, true),
        (Feature::AccountDeletion, false, false, true),
        (Feature::Drive, false, false, true),
        (Feature::DriveShare, false, false, true),
        (Feature::DriveAttach, false, false, true),
        (Feature::FeedAccounts, false, false, false),
        (Feature::Events, false, false, false),
        (Feature::SyncToken, false, false, false),
        (Feature::Favourites, false, true, true),
    ];

    #[test]
    fn the_matrix_pins_every_row_as_probed() {
        assert_eq!(EXPECTED.len(), Feature::COUNT);
        let nodus = active(Provider::Nodus);
        let mastodon = active(Provider::Mastodon);
        let misskey = active(Provider::Misskey);
        for &(feature, in_nodus, in_mastodon, in_misskey) in EXPECTED {
            assert_eq!(nodus.contains(feature), in_nodus, "nodus {feature:?}");
            assert_eq!(
                mastodon.contains(feature),
                in_mastodon,
                "mastodon {feature:?}"
            );
            assert_eq!(misskey.contains(feature), in_misskey, "misskey {feature:?}");
        }
    }

    #[test]
    fn the_three_protocol_sets_are_decided_for_every_feature() {
        let nodus = active(Provider::Nodus);
        let mastodon = active(Provider::Mastodon);
        let misskey = active(Provider::Misskey);
        for feature in Feature::ALL {
            let row = decision(feature);
            assert_eq!(nodus.contains(feature), row[0], "nodus {feature:?}");
            assert_eq!(mastodon.contains(feature), row[1], "mastodon {feature:?}");
            assert_eq!(misskey.contains(feature), row[2], "misskey {feature:?}");
        }
    }

    #[test]
    fn nodus_must_publish_because_its_baseline_is_empty() {
        assert!(active(Provider::Nodus).is_empty());
    }

    #[test]
    fn favourites_is_served_by_mastodon_and_misskey() {
        assert!(active(Provider::Mastodon).contains(Feature::Favourites));
        assert!(active(Provider::Misskey).contains(Feature::Favourites));
    }
}
