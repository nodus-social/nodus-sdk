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
            Self::AdminOverview => "admin_overview",
            Self::AdminAnnouncements => "admin_announcements",
            Self::AdminAudit => "admin_audit",
            Self::AdminDomains => "admin_domains",
            Self::AdminEmail => "admin_email",
            Self::AdminEscalation => "admin_escalation",
            Self::AdminFederation => "admin_federation",
            Self::AdminProtocol => "admin_protocol",
            Self::AdminReplies => "admin_replies",
            Self::AdminRules => "admin_rules",
            Self::AdminScanners => "admin_scanners",
            Self::AdminSettings => "admin_settings",
            Self::AdminTopology => "admin_topology",
            Self::ModerationAccounts => "moderation_accounts",
            Self::ModerationAppeals => "moderation_appeals",
            Self::ModerationFederation => "moderation_federation",
            Self::ModerationFlags => "moderation_flags",
            Self::ModerationMedia => "moderation_media",
            Self::ModerationRegistrations => "moderation_registrations",
            Self::ModerationReports => "moderation_reports",
            Self::Mutes => "mutes",
            Self::Blocks => "blocks",
            Self::DomainBlocks => "domain_blocks",
            Self::Sessions => "sessions",
            Self::TwoFactor => "two_factor",
            Self::Exports => "exports",
            Self::FeaturedTags => "featured_tags",
            Self::NotificationSettings => "notification_settings",
            Self::Pages => "pages",
            Self::StylePublic => "style_public",
            Self::Push => "push",
            Self::MediaLibrary => "media_library",
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
