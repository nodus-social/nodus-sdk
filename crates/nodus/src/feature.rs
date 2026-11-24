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
    /// Administration: the dashboard of counts and pending decisions.
    AdminOverview,
    /// Administration: instance announcements and their publish controls.
    AdminAnnouncements,
    /// Administration: the log of actions staff took.
    AdminAudit,
    /// Administration: the domains this instance allows in.
    AdminDomains,
    /// Administration: mail domains, i.e. which addresses may sign up.
    AdminEmail,
    /// Administration: the escalation policy and its ladder of strikes.
    AdminEscalation,
    /// Administration: federation deliveries still owed to peers, and why they stopped.
    AdminFederation,
    /// Administration: what protocol the server speaks, as the client detected it.
    AdminProtocol,
    /// Administration: canned replies (Mastodon calls them warning presets).
    AdminReplies,
    /// Administration: the instance's published rules and their order.
    AdminRules,
    /// Administration: content scanners and their label thresholds.
    AdminScanners,
    /// Administration: instance settings such as branding and registration policy.
    AdminSettings,
    /// Administration: the instance's own processes and their recent logs.
    AdminTopology,
    /// Moderation: account standing and the decisions a moderator can take.
    ModerationAccounts,
    /// Moderation: appeals against a decision about a person.
    ModerationAppeals,
    /// Moderation: domains this instance refuses.
    ModerationFederation,
    /// Moderation: the queue of content a scanner flagged for a person to judge.
    ModerationFlags,
    /// Moderation: quarantined media awaiting release or deletion.
    ModerationMedia,
    /// Moderation: who may join and who is waiting to be admitted.
    ModerationRegistrations,
    /// Moderation: the report queue.
    ModerationReports,
    /// The accounts this account mutes (`GET /api/v1/mutes`).
    Mutes,
    /// The accounts this account blocks (`GET /api/v1/blocks`).
    Blocks,
    /// Domains this account mutes or blocks (`/api/v1/domain_blocks`).
    DomainBlocks,
    /// Signed-in sessions this account can review and revoke.
    Sessions,
    /// Two-factor authentication enrolment, keys and recovery codes.
    TwoFactor,
    /// Downloading the account's own data (`/settings/export`, `i/export-*`).
    Exports,
    /// Tags an account features on its profile (`/api/v1/featured_tags`).
    FeaturedTags,
    /// Which notifications an account wants (`/api/v1/notifications/policy`).
    NotificationSettings,
    /// A page-shaped read of a profile or thread in one payload
    /// (`GET /api/v1/pages/profile/:id`, `GET /api/v1/pages/status/:id` on Nodus).
    Pages,
    /// Publishing the banners a profile shows to other readers
    /// (`PUT /api/v1/style/public`, read at
    /// `GET /api/v1/accounts/:id/style_public` on Nodus).
    StylePublic,
    /// Web Push subscription for the browser (`/api/v1/push/subscription`).
    Push,
    /// Media library with usage tracking and safe delete (`/api/v1/media/library` on Nodus).
    MediaLibrary,
}

impl Feature {
    /// How many capabilities this build names.
    ///
    /// Stated separately from [`Feature::ALL`] so the two must agree; the assertion
    /// below turns a forgotten row into a compile error rather than a missing bit.
    pub const COUNT: usize = 74;

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
        Self::AdminOverview,
        Self::AdminAnnouncements,
        Self::AdminAudit,
        Self::AdminDomains,
        Self::AdminEmail,
        Self::AdminEscalation,
        Self::AdminFederation,
        Self::AdminProtocol,
        Self::AdminReplies,
        Self::AdminRules,
        Self::AdminScanners,
        Self::AdminSettings,
        Self::AdminTopology,
        Self::ModerationAccounts,
        Self::ModerationAppeals,
        Self::ModerationFederation,
        Self::ModerationFlags,
        Self::ModerationMedia,
        Self::ModerationRegistrations,
        Self::ModerationReports,
        Self::Mutes,
        Self::Blocks,
        Self::DomainBlocks,
        Self::Sessions,
        Self::TwoFactor,
        Self::Exports,
        Self::FeaturedTags,
        Self::NotificationSettings,
        Self::Pages,
        Self::StylePublic,
        Self::Push,
        Self::MediaLibrary,
    ];
}

mod index;
mod wire;

const _: () = assert!(Feature::COUNT == Feature::ALL.len());

#[cfg(test)]
mod tests;
