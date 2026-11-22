//! The bit index of each capability.

use super::Feature;

impl Feature {
    /// Position in declaration order, used as the bit index in [`ActiveFeatures`].
    ///
    /// An exhaustive match rather than `self as usize`: a new variant must be given a
    /// slot explicitly, and it cannot silently alias an existing feature's bit.
    ///
    /// [`ActiveFeatures`]: crate::ActiveFeatures
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Admin => 0,
            Self::Moderation => 1,
            Self::Scanners => 2,
            Self::Topology => 3,
            Self::Protocol => 4,
            Self::RemoteBlocks => 5,
            Self::Polls => 6,
            Self::Batch => 7,
            Self::Appeals => 8,
            Self::InstanceRules => 9,
            Self::ModerationRules => 10,
            Self::MediaQuarantine => 11,
            Self::BlockLists => 12,
            Self::ModerationStats => 13,
            Self::RemoteAccounts => 14,
            Self::Warnings => 15,
            Self::Escalation => 16,
            Self::FederationDelivery => 17,
            Self::DomainPauses => 18,
            Self::SettingsTransfer => 19,
            Self::ReportForwarding => 20,
            Self::AccountDeletion => 21,
            Self::Drive => 22,
            Self::DriveShare => 23,
            Self::DriveAttach => 24,
            Self::FeedAccounts => 25,
            Self::Events => 26,
            Self::SyncToken => 27,
            Self::Favourites => 28,
            Self::Bookmarks => 29,
            Self::Conversations => 30,
            Self::Lists => 31,
            Self::Filters => 32,
            Self::ScheduledStatuses => 33,
            Self::Notifications => 34,
            Self::Announcements => 35,
            Self::Search => 36,
            Self::Directory => 37,
            Self::Markers => 38,
            Self::Preferences => 39,
            Self::Streaming => 40,
            Self::Tags => 41,
        }
    }
}
