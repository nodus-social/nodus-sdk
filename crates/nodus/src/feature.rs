//! The capability vocabulary shared by every protocol the SDK speaks.
//!
//! One name describes one surface, so a screen can ask "does this server serve
//! polls?" without branching on whether it is talking to Nodus, Mastodon or Misskey.
//! The spelling is the one `GET /api/v1/meta` publishes in `features`, which lets the
//! same word describe a Nodus extension and a baseline surface elsewhere.

use serde::{Deserialize, Serialize};

/// A capability a server may implement.
///
/// Deliberately a closed set: a free-form string would let two servers claim the same
/// word for different behaviour, and the client cannot defend against that. A name this
/// build does not know is not discarded — [`Capabilities`](crate::Capabilities) keeps it
/// in `unknown`, so a newer server still informs an older client, and an unknown local
/// name simply parses to `None` rather than failing the whole answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    /// Instance administration (`/api/v1/admin/*`).
    Admin,
    /// Moderation queue and actions (`/api/v1/moderation/*` on Nodus).
    Moderation,
    /// Account and content scanners (`/api/v1/scanners/*` on Nodus).
    Scanners,
    /// Federation topology inspection (`/api/v1/topology` on Nodus).
    Topology,
    /// Capability negotiation itself (`/api/v1/meta` on Nodus).
    Protocol,
    /// Remote-instance block management.
    RemoteBlocks,
    /// Poll creation and voting (`/api/v1/polls/:id/votes`).
    Polls,
    /// Several reads in one round trip (`POST /api/v1/batch`).
    Batch,
    /// Appeals against a moderation decision (`/api/v1/appeals`).
    Appeals,
    /// Instance rules (`/api/v1/instance/rules`).
    InstanceRules,
    /// Moderation rules, separate from the rules shown on the instance page.
    ModerationRules,
    /// Quarantine of uploaded media pending review.
    MediaQuarantine,
    /// Shared block lists.
    BlockLists,
    /// Moderation statistics.
    ModerationStats,
    /// Remote-account lookup and administration.
    RemoteAccounts,
    /// Moderation warnings shown to the account they concern.
    Warnings,
    /// Escalation policy for reports.
    Escalation,
    /// Federation delivery controls.
    FederationDelivery,
    /// Pauses on delivery to one domain.
    DomainPauses,
    /// Export and import of client settings.
    SettingsTransfer,
    /// Forwarding of reports to the reported account's instance.
    ReportForwarding,
    /// Account deletion from the client.
    AccountDeletion,
    /// File storage (Misskey Drive).
    Drive,
    /// Link-sharing a stored file.
    DriveShare,
    /// Attaching a stored file to a post.
    DriveAttach,
    /// `GET /api/v1/feed/accounts`: recommendations, server-side.
    FeedAccounts,
    /// `POST /api/v1/events`: interaction feedback is accepted.
    Events,
    /// `PUT /api/v1/sync/token`: the server keeps an encrypted token and syncs follows.
    SyncToken,
    /// `GET /api/v1/favourites`: the account's saved posts.
    Favourites,
}

impl Feature {
    /// Every capability this build names.
    ///
    /// A table rather than a match in [`Feature::parse`], so adding a variant is one
    /// edit and a forgotten spelling cannot silently become "unknown".
    pub const ALL: [Self; 29] = [
        Self::Admin,
        Self::Moderation,
        Self::Scanners,
        Self::Topology,
        Self::Protocol,
        Self::RemoteBlocks,
        Self::Polls,
        Self::Batch,
        Self::Appeals,
        Self::InstanceRules,
        Self::ModerationRules,
        Self::MediaQuarantine,
        Self::BlockLists,
        Self::ModerationStats,
        Self::RemoteAccounts,
        Self::Warnings,
        Self::Escalation,
        Self::FederationDelivery,
        Self::DomainPauses,
        Self::SettingsTransfer,
        Self::ReportForwarding,
        Self::AccountDeletion,
        Self::Drive,
        Self::DriveShare,
        Self::DriveAttach,
        Self::FeedAccounts,
        Self::Events,
        Self::SyncToken,
        Self::Favourites,
    ];

    /// The wire spelling, which is also what the server advertises.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Moderation => "moderation",
            Self::Scanners => "scanners",
            Self::Topology => "topology",
            Self::Protocol => "protocol",
            Self::RemoteBlocks => "remote_blocks",
            Self::Polls => "polls",
            Self::Batch => "batch",
            Self::Appeals => "appeals",
            Self::InstanceRules => "instance_rules",
            Self::ModerationRules => "moderation_rules",
            Self::MediaQuarantine => "media_quarantine",
            Self::BlockLists => "block_lists",
            Self::ModerationStats => "moderation_stats",
            Self::RemoteAccounts => "remote_accounts",
            Self::Warnings => "warnings",
            Self::Escalation => "escalation",
            Self::FederationDelivery => "federation_delivery",
            Self::DomainPauses => "domain_pauses",
            Self::SettingsTransfer => "settings_transfer",
            Self::ReportForwarding => "report_forwarding",
            Self::AccountDeletion => "account_deletion",
            Self::Drive => "drive",
            Self::DriveShare => "drive_share",
            Self::DriveAttach => "drive_attach",
            Self::FeedAccounts => "feed_accounts",
            Self::Events => "events",
            Self::SyncToken => "sync_token",
            Self::Favourites => "favourites",
        }
    }

    /// Parses a wire spelling; an unknown name is `None`, never an error.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|feature| feature.as_str() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_feature_spelling_round_trips() {
        for feature in Feature::ALL {
            assert_eq!(Feature::parse(feature.as_str()), Some(feature));
        }
    }

    #[test]
    fn an_unknown_spelling_is_not_a_feature_rather_than_an_error() {
        assert_eq!(Feature::parse("telepathy"), None);
    }

    #[test]
    fn the_names_nodus_publishes_all_parse() {
        let published = [
            "admin",
            "moderation",
            "scanners",
            "topology",
            "protocol",
            "remote_blocks",
            "polls",
            "batch",
            "appeals",
            "instance_rules",
            "moderation_rules",
            "media_quarantine",
            "block_lists",
            "moderation_stats",
            "remote_accounts",
            "warnings",
            "escalation",
            "federation_delivery",
            "domain_pauses",
            "settings_transfer",
            "report_forwarding",
            "account_deletion",
            "drive",
            "drive_share",
            "drive_attach",
            "feed_accounts",
            "events",
            "sync_token",
        ];
        for name in published {
            assert!(Feature::parse(name).is_some(), "{name} did not parse");
        }
    }
}
