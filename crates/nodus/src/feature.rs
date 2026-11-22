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
#[repr(u8)]
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
    /// `GET /api/v1/bookmarks`: posts saved for later rather than starred.
    Bookmarks,
    /// Direct-message threads (`GET /api/v1/conversations`).
    Conversations,
    /// Curated timelines (`/api/v1/lists`).
    Lists,
    /// Keyword filters applied to timelines and notifications (`/api/v2/filters`).
    Filters,
    /// Posts queued for a later publish time (`/api/v1/scheduled_statuses`).
    ScheduledStatuses,
    /// The notification inbox (`/api/v1/notifications`).
    Notifications,
    /// Instance announcements (`/api/v1/announcements`).
    Announcements,
    /// Full-text and account search (`/api/v2/search`).
    Search,
    /// The local profile directory (`/api/v1/directory`).
    Directory,
    /// Read positions per timeline (`/api/v1/markers`).
    Markers,
    /// Server-held posting defaults (`/api/v1/preferences`).
    Preferences,
    /// Live updates over the streaming API (`/api/v1/streaming`).
    Streaming,
    /// Tag pages and following a tag (`/api/v1/tags/:name`).
    Tags,
}

impl Feature {
    /// How many capabilities this build names.
    ///
    /// Stated separately from [`Feature::ALL`] so the two must agree; the assertion
    /// below turns a forgotten row into a compile error rather than a missing bit.
    pub const COUNT: usize = 42;

    /// Every capability this build names.
    ///
    /// A table rather than a match in [`Feature::parse`], so adding a variant is one
    /// edit and a forgotten spelling cannot silently become "unknown".
    pub const ALL: [Self; Self::COUNT] = [
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
        Self::Bookmarks,
        Self::Conversations,
        Self::Lists,
        Self::Filters,
        Self::ScheduledStatuses,
        Self::Notifications,
        Self::Announcements,
        Self::Search,
        Self::Directory,
        Self::Markers,
        Self::Preferences,
        Self::Streaming,
        Self::Tags,
    ];
}

mod index;
mod wire;

const _: () = assert!(Feature::COUNT == Feature::ALL.len());

#[cfg(test)]
mod tests;
