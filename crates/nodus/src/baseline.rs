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
//! ## Admin pages with no API are declared
//!
//! An administration page rendered by the server's own HTML frontend has no client API to
//! probe, so its row says `DECLARED`, names the page, and cites the server's route table.
//! Declaring such a row `true` is what lets the client draw the screen; the alternative —
//! probing an invented API path, getting a `404`, and calling the page absent — would
//! hide a page the server does serve. `admin_protocol` is different: that page is
//! client-side detection with no server route at all, so it stays `false` everywhere.
//!
//! The per-feature evidence, including the status code behind each decision, lives in
//! the `decision` module, next to the table it justifies.
//!
//! Everything Nodus adds beyond a protocol's own client API stays `false`.

use crate::active::ActiveFeatures;
use crate::capabilities::Provider;
use crate::feature::Feature;

mod decision;

use decision::decision;

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
        (Feature::Bookmarks, false, true, false),
        (Feature::Conversations, false, true, false),
        (Feature::Lists, false, true, true),
        (Feature::Filters, false, true, false),
        (Feature::ScheduledStatuses, false, true, false),
        (Feature::Notifications, false, true, true),
        (Feature::Announcements, false, true, true),
        (Feature::Search, false, true, true),
        (Feature::Directory, false, true, true),
        (Feature::Markers, false, true, false),
        (Feature::Preferences, false, true, false),
        (Feature::Streaming, false, true, true),
        (Feature::Tags, false, true, false),
        (Feature::AdminOverview, false, true, true),
        (Feature::AdminAnnouncements, false, true, true),
        (Feature::AdminAudit, false, true, true),
        (Feature::AdminDomains, false, true, false),
        (Feature::AdminEmail, false, true, false),
        (Feature::AdminEscalation, false, false, false),
        (Feature::AdminFederation, false, false, true),
        (Feature::AdminProtocol, false, false, false),
        (Feature::AdminReplies, false, true, false),
        (Feature::AdminRules, false, true, true),
        (Feature::AdminScanners, false, false, false),
        (Feature::AdminSettings, false, true, true),
        (Feature::AdminTopology, false, false, false),
        (Feature::ModerationAccounts, false, true, true),
        (Feature::ModerationAppeals, false, true, false),
        (Feature::ModerationFederation, false, true, true),
        (Feature::ModerationFlags, false, false, false),
        (Feature::ModerationMedia, false, false, false),
        (Feature::ModerationRegistrations, false, true, true),
        (Feature::ModerationReports, false, true, true),
        (Feature::Mutes, false, true, true),
        (Feature::Blocks, false, true, true),
        (Feature::DomainBlocks, false, true, true),
        (Feature::Sessions, false, true, false),
        (Feature::TwoFactor, false, true, true),
        (Feature::Exports, false, true, true),
        (Feature::FeaturedTags, false, true, false),
        (Feature::NotificationSettings, false, true, false),
        (Feature::Pages, false, false, false),
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

    #[test]
    fn misskey_bookmarks_are_absent_while_its_favourites_are_served() {
        let misskey = active(Provider::Misskey);
        assert!(misskey.contains(Feature::Favourites));
        assert!(!misskey.contains(Feature::Bookmarks));
        assert!(active(Provider::Mastodon).contains(Feature::Bookmarks));
    }

    #[test]
    fn misskey_serves_the_collections_it_has_routes_for() {
        let misskey = active(Provider::Misskey);
        for feature in [
            Feature::Lists,
            Feature::Notifications,
            Feature::Announcements,
            Feature::Search,
            Feature::Directory,
            Feature::Streaming,
        ] {
            assert!(misskey.contains(feature), "misskey {feature:?}");
        }
        for feature in [
            Feature::Conversations,
            Feature::Filters,
            Feature::ScheduledStatuses,
            Feature::Markers,
            Feature::Preferences,
            Feature::Tags,
        ] {
            assert!(!misskey.contains(feature), "misskey {feature:?}");
        }
    }
}
