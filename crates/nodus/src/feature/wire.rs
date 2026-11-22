//! The wire spelling of each capability.

use super::Feature;

impl Feature {
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
            Self::Bookmarks => "bookmarks",
            Self::Conversations => "conversations",
            Self::Lists => "lists",
            Self::Filters => "filters",
            Self::ScheduledStatuses => "scheduled_statuses",
            Self::Notifications => "notifications",
            Self::Announcements => "announcements",
            Self::Search => "search",
            Self::Directory => "directory",
            Self::Markers => "markers",
            Self::Preferences => "preferences",
            Self::Streaming => "streaming",
            Self::Tags => "tags",
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
