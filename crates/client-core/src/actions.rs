//! The verbs a full client needs, grouped by what they act on.
//!
//! [`InstanceApi`](crate::api::InstanceApi) is the *read* surface the SDK state machine
//! drives: timelines, one status, notifications. Everything a user does — favouriting,
//! boosting, voting, following, blocking — is an action, and actions are not state: the
//! caller decides when to run one and what to do with the answer. Keeping them out of
//! `InstanceApi` means a fake instance in a test implements three reads, not thirty
//! methods, and it keeps the trait a description of what the SDK needs rather than a
//! description of Mastodon.
//!
//! Every method returns the object the instance returns, not `()`, because the client
//! renders the answer: an optimistic update followed by a silent failure is worse than
//! not updating at all.
//!
//! Follow, unfollow and delete already exist on
//! [`InstanceApi`](crate::api::InstanceApi) — they predate this split because the
//! recommender's sync worker needs them — so they are deliberately absent here. Two
//! traits declaring one method would make every call site ambiguous.

use core::fmt::Debug;
use core::future::Future;

use crate::model::{
    Account, AccountDetail, AccountNote, AdminAccount, AdminAccountQuery, AdminAlert, AdminMeasure,
    AdminMeta, AdminOverview, Announcement, AnnouncementDraft, AssetUpload, AuditEntry,
    AuditFilter, Conversation, CustomEmoji, DomainAllow, DomainBlock, DomainBlockDraft,
    DomainImportOutcome, DomainSeverity, Draft, EmailDomainBlock, FeaturedTag, Filter, FilterDraft,
    FilterV1, FilterV1Draft, FlagState, HistoryEntry, InstanceInfo, Language, List, LocalId,
    MarkerTimeline, MarkerUpdate, Markers, MediaAttachment, ModerationAction, NewScanner, NodeLog,
    Page, PageParams, Poll, Preferences, Relationship, Report, ReportDraft, ReportFilter,
    ReportState, Rule, Scan, ScanFlag, ScanRequest, ScanSeverity, Scanner, ScannerAsset,
    ScannerRule, ScheduledStatus, SearchResults, Status, Suggestion, Tag, Topology, TrendingLink,
    TriggerOutcome, WarningPreset,
};

/// Things done to a status.
pub trait StatusActions {
    /// Error type of the transport.
    type Error: Debug;

    /// Favourites a status, returning it as the instance now reports it.
    fn favourite(&self, id: &str) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Removes the signed-in account's favourite.
    fn unfavourite(&self, id: &str) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Boosts a status.
    fn reblog(&self, id: &str) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Removes the boost.
    fn unreblog(&self, id: &str) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Bookmarks a status; private to the account, so no other instance learns it.
    fn bookmark(&self, id: &str) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Removes the bookmark.
    fn unbookmark(&self, id: &str) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Pins a status to the account's profile.
    fn pin(&self, id: &str) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Unpins it.
    fn unpin(&self, id: &str) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Edits a status, keeping its id and adding a revision.
    fn edit_status(
        &self,
        id: &str,
        draft: &Draft,
    ) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Votes on an attached poll.
    fn vote(
        &self,
        poll_id: &str,
        choices: &[usize],
    ) -> impl Future<Output = Result<Poll, Self::Error>>;

    /// The edit history, oldest first.
    fn history(&self, id: &str) -> impl Future<Output = Result<Vec<HistoryEntry>, Self::Error>>;

    /// Accounts that boosted the status.
    fn reblogged_by(
        &self,
        id: &str,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Account>, Self::Error>>;

    /// Accounts that favourited the status.
    fn favourited_by(
        &self,
        id: &str,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Account>, Self::Error>>;
}

/// Things done to an account.
pub trait MediaActions {
    type Error: Debug;

    /// Uploads one attachment and returns what the instance made of it.
    ///
    /// The bytes are passed as a slice because the caller owns the file object: the SDK
    /// never opens a path, web or native.
    fn upload_media(
        &self,
        bytes: &[u8],
        mime: &str,
        description: Option<&str>,
    ) -> impl Future<Output = Result<MediaAttachment, Self::Error>>;

    /// Sets or replaces one attachment's description.
    ///
    /// Separate from [`Self::upload_media`] because alt text is often written after the
    /// picture is already attached, and a caller that can only upload would have to send
    /// the bytes again to say what they show.
    fn update_media(
        &self,
        id: &str,
        description: &str,
    ) -> impl Future<Output = Result<MediaAttachment, Self::Error>>;

    /// Marks one attachment as sensitive without rewriting its description.
    fn set_media_sensitive(
        &self,
        id: &str,
        sensitive: bool,
    ) -> impl Future<Output = Result<MediaAttachment, Self::Error>>;

    /// Reads the processing state of one attachment.
    fn media_status(&self, id: &str) -> impl Future<Output = Result<MediaAttachment, Self::Error>>;
}

pub trait AccountActions {
    /// Error type of the transport.
    type Error: Debug;

    /// Blocks: no follows in either direction, no mentions.
    fn block(&self, id: &LocalId) -> impl Future<Output = Result<Relationship, Self::Error>>;

    /// Lifts the block.
    fn unblock(&self, id: &LocalId) -> impl Future<Output = Result<Relationship, Self::Error>>;

    /// Mutes: their posts leave every timeline, they are not told.
    fn mute(&self, id: &LocalId) -> impl Future<Output = Result<Relationship, Self::Error>>;

    /// Lifts the mute.
    fn unmute(&self, id: &LocalId) -> impl Future<Output = Result<Relationship, Self::Error>>;

    /// Relationships for several accounts in one call.
    ///
    /// Returns them in the order asked; an instance may answer fewer, which the caller
    /// reads as "unknown" rather than as "not following".
    fn relationships(
        &self,
        ids: &[LocalId],
    ) -> impl Future<Output = Result<Vec<Relationship>, Self::Error>>;

    /// Who follows `id`.
    fn followers(
        &self,
        id: &LocalId,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Account>, Self::Error>>;

    /// Who `id` follows.
    fn follow_requests(&self) -> impl Future<Output = Result<Vec<Account>, Self::Error>>;

    /// Accepts a pending follow request.
    fn authorize_follow_request(
        &self,
        id: &LocalId,
    ) -> impl Future<Output = Result<Relationship, Self::Error>>;

    /// Refuses a pending follow request.
    fn reject_follow_request(&self, id: &LocalId) -> impl Future<Output = Result<(), Self::Error>>;

    fn mutes(&self, params: PageParams)
    -> impl Future<Output = Result<Page<Account>, Self::Error>>;

    fn blocks(
        &self,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Account>, Self::Error>>;

    fn following(
        &self,
        id: &LocalId,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Account>, Self::Error>>;

    /// Changes the signed-in account's own profile.
    ///
    /// Answers the account as the instance now holds it, so the caller renders the
    /// sanitised result rather than the text it sent.
    fn update_profile(
        &self,
        update: &crate::model::ProfileUpdate,
    ) -> impl Future<Output = Result<Account, Self::Error>>;
}

/// Reading positions, one per timeline.
///
/// The server keeps this so a second device opens where the first one stopped; a client
/// that only stores positions locally loses them on every reinstall and re-notifies for
/// posts already read elsewhere.
pub trait MarkerActions {
    /// Error type of the transport.
    type Error: Debug;

    /// The markers for `timelines`, as the instance holds them.
    fn markers(
        &self,
        timelines: &[MarkerTimeline],
    ) -> impl Future<Output = Result<Markers, Self::Error>>;

    /// Writes reading positions, answering with the merged set.
    ///
    /// A write whose `version` is older than the stored one is ignored by the instance,
    /// so the answer is what the caller should render, not what it sent.
    fn save_markers(
        &self,
        updates: &[MarkerUpdate],
    ) -> impl Future<Output = Result<Markers, Self::Error>>;
}

/// The collections a client lists: favourites, bookmarks, conversations, lists.
pub trait Browse {
    /// Error type of the transport.
    type Error: Debug;

    /// Statuses the account favourited, newest first.
    fn favourites(
        &self,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Status>, Self::Error>>;

    /// Statuses the account bookmarked, newest first.
    fn bookmarks(
        &self,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Status>, Self::Error>>;

    /// Direct-message threads, newest first.
    fn conversations(
        &self,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Conversation>, Self::Error>>;

    /// Marks a conversation as read.
    fn mark_conversation_read(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// How many accounts one directory page holds.
    ///
    /// Published because the directory is paged by offset: a caller that wants the next
    /// page needs to know when the previous one was full, and Mastodon answers with a
    /// bare array rather than a cursor.
    const DIRECTORY_PAGE: u32 = 40;

    /// Accounts the instance suggests, for finding people to follow.
    fn directory(
        &self,
        local_only: bool,
        offset: u32,
    ) -> impl Future<Output = Result<Vec<Account>, Self::Error>>;

    /// Accounts the instance's suggestion algorithm ranks highest.
    ///
    /// The v2 surface rather than the retired v1 one: it names the reason an account was
    /// suggested, which is what lets a reader decide whether to trust the list.
    fn suggestions(&self, limit: u16)
    -> impl Future<Output = Result<Vec<Suggestion>, Self::Error>>;

    /// Mutes or unmutes a conversation.
    fn mute_conversation(
        &self,
        id: &str,
        muted: bool,
    ) -> impl Future<Output = Result<Conversation, Self::Error>>;

    /// The account's lists.
    fn lists(&self) -> impl Future<Output = Result<Vec<List>, Self::Error>>;

    /// Creates a list.
    fn create_list(&self, title: &str) -> impl Future<Output = Result<List, Self::Error>>;

    /// Renames a list.
    fn rename_list(&self, id: &str, title: &str)
    -> impl Future<Output = Result<List, Self::Error>>;

    /// Deletes a list; its members stay followed.
    fn delete_list(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// Adds accounts to a list.
    fn add_to_list(
        &self,
        list_id: &str,
        accounts: &[LocalId],
    ) -> impl Future<Output = Result<(), Self::Error>>;

    /// Removes accounts from a list.
    fn remove_from_list(
        &self,
        list_id: &str,
        accounts: &[LocalId],
    ) -> impl Future<Output = Result<(), Self::Error>>;

    /// The accounts on a list.
    ///
    /// Mastodon returns the whole membership, not a page: a list is a hand-picked set
    /// of people, and the client needs all of them to answer "is this account on it".
    fn list_accounts(
        &self,
        list_id: &str,
    ) -> impl Future<Output = Result<Vec<Account>, Self::Error>>;

    /// A list's timeline.
    ///
    /// Same cursor rules as the other timelines — a full page is what produces the next
    /// id — because it is the same endpoint shape and the same UI below it.
    fn list_timeline(
        &self,
        list_id: &str,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Status>, Self::Error>>;
}

/// Searching the instance and, when `resolve` is set, the fediverse.
pub trait Search {
    /// Error type of the transport.
    type Error: Debug;

    /// Searches accounts, statuses and hashtags.
    ///
    /// `resolve` is the difference between "what this instance already knows" and
    /// "go and fetch it, which may take seconds and may find nothing": a search-as-you-
    /// type box wants the former, a deliberate lookup the latter.
    fn search(
        &self,
        query: &str,
        resolve: bool,
    ) -> impl Future<Output = Result<SearchResults, Self::Error>>;
}

/// The reader's own filters.
///
/// A filter is not a status action: it changes what a *timeline* shows, and the server
/// already applies it. This surface exists so the client can edit the list and, for a
/// thread assembled from cached statuses, apply the same rule itself.
pub trait FilterActions {
    /// Error type of the transport.
    type Error: Debug;

    /// Every filter this account keeps.
    fn filters(&self) -> impl Future<Output = Result<Vec<Filter>, Self::Error>>;

    /// Create a filter.
    fn create_filter(
        &self,
        draft: &FilterDraft,
    ) -> impl Future<Output = Result<Filter, Self::Error>>;

    /// Replace a filter's settings and phrases.
    fn update_filter(
        &self,
        id: &str,
        draft: &FilterDraft,
    ) -> impl Future<Output = Result<Filter, Self::Error>>;

    /// Delete a filter and its phrases.
    fn delete_filter(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;
}

/// The filters older instances keep, under `/api/v1/filters`.
///
/// Mastodon 4.0 replaced this surface with v2's keyword lists, but instances before 4.0
/// serve only this one, and later ones still accept it. A filter here has a single
/// `phrase` and an `irreversible` flag instead of keywords and an action.
pub trait FilterV1Actions {
    /// Error type of the transport.
    type Error: Debug;

    /// Every v1 filter this account keeps.
    fn filters_v1(&self) -> impl Future<Output = Result<Vec<FilterV1>, Self::Error>>;

    /// Create a v1 filter.
    fn create_filter_v1(
        &self,
        draft: &FilterV1Draft,
    ) -> impl Future<Output = Result<FilterV1, Self::Error>>;

    /// Replace a v1 filter's phrase and settings.
    fn update_filter_v1(
        &self,
        id: &str,
        draft: &FilterV1Draft,
    ) -> impl Future<Output = Result<FilterV1, Self::Error>>;

    /// Delete a v1 filter.
    fn delete_filter_v1(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;
}

/// Moderation: the report queue, account standing, blocked domains.
///
/// Every call here needs `admin:read` or `admin:write`, so a client must ask before it
/// renders a screen: an ordinary token gets a 403, and a menu that leads to a 403 is
/// worse than a menu that is not there.
pub trait AdminActions {
    /// Error type of the transport.
    type Error: Debug;

    /// Reports waiting for a decision, or already decided. Requires `admin:read`.
    fn reports(
        &self,
        state: ReportState,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<Report>, Self::Error>>;

    /// One page of the queue, narrowed by whatever the filter names. Requires `admin:read`.
    fn reports_page(
        &self,
        filter: &ReportFilter,
    ) -> impl Future<Output = Result<Vec<Report>, Self::Error>>;

    /// One report with its accounts and statuses filled in. Requires `admin:read`.
    fn report(&self, id: &str) -> impl Future<Output = Result<Report, Self::Error>>;

    /// Mark a report as handled. Requires `admin:write`.
    fn resolve_report(&self, id: &str) -> impl Future<Output = Result<Report, Self::Error>>;

    /// Mark a report as handled, recording why.
    ///
    /// The note is the decision's only memory once the queue has moved on, which is why it
    /// travels with the resolve rather than in a follow-up note. Requires `admin:write`.
    fn resolve_report_with_note(
        &self,
        id: &str,
        note: &str,
    ) -> impl Future<Output = Result<Report, Self::Error>>;

    /// Reopen a handled report. Requires `admin:write`.
    fn reopen_report(&self, id: &str) -> impl Future<Output = Result<Report, Self::Error>>;

    /// Claim a report for `account_id`, or pass `None` to put it back in the unclaimed
    /// queue. Requires `admin:write`.
    fn assign_report(
        &self,
        id: &str,
        account_id: Option<&str>,
    ) -> impl Future<Output = Result<Report, Self::Error>>;

    /// File a report. Any signed-in account may do this; no admin scope is needed.
    fn create_report(
        &self,
        draft: &ReportDraft,
    ) -> impl Future<Output = Result<Report, Self::Error>>;

    /// Accounts, newest first, optionally only local ones. Requires `admin:read`.
    fn admin_accounts(
        &self,
        local: bool,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<AdminAccount>, Self::Error>>;

    /// The account directory, narrowed by origin, standing, name, mail or domain.
    /// Requires `admin:read`.
    fn search_accounts(
        &self,
        query: &AdminAccountQuery,
    ) -> impl Future<Output = Result<Vec<AdminAccount>, Self::Error>>;

    /// Apply a decision to one account. Requires `admin:write`.
    fn moderate_account(
        &self,
        id: &str,
        action: ModerationAction,
    ) -> impl Future<Output = Result<AdminAccount, Self::Error>>;

    /// Apply a decision, recording why and optionally resolving the report behind it.
    ///
    /// One call rather than three, because a moderator who silences an account and then
    /// fails to write the note has made a decision nobody can review. Requires
    /// `admin:write`.
    fn moderate_account_with_note(
        &self,
        id: &str,
        action: ModerationAction,
        note: &str,
        report_id: Option<&str>,
    ) -> impl Future<Output = Result<AdminAccount, Self::Error>>;

    /// Grant or revoke administrator rights. Requires `admin:write`.
    fn set_staff(
        &self,
        id: &str,
        staff: bool,
    ) -> impl Future<Output = Result<AdminAccount, Self::Error>>;

    /// One account with its notes, scanner flags and report count. Requires `admin:read`.
    fn account_detail(&self, id: &str) -> impl Future<Output = Result<AccountDetail, Self::Error>>;

    /// Add a private note to an account. Requires `admin:write`.
    fn add_account_note(
        &self,
        id: &str,
        body: &str,
    ) -> impl Future<Output = Result<AccountNote, Self::Error>>;

    /// Delete one of an account's notes. Requires `admin:write`.
    fn delete_account_note(
        &self,
        id: &str,
        note_id: &str,
    ) -> impl Future<Output = Result<(), Self::Error>>;

    /// Delete the account and everything it owns. Irreversible, so the caller must mean
    /// it: the server refuses the signed-in account and any administrator.
    /// Requires `admin:write`.
    fn purge_account(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// Allow or refuse lossless uploads from one account. Requires `admin:write`.
    fn set_lossless(
        &self,
        id: &str,
        images: bool,
        videos: bool,
    ) -> impl Future<Output = Result<(), Self::Error>>;

    /// Domains the instance refuses. Requires `admin:read`.
    fn domain_blocks(
        &self,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<DomainBlock>, Self::Error>>;

    /// Refuse a domain. Requires `admin:write`.
    fn create_domain_block(
        &self,
        domain: &str,
        severity: DomainSeverity,
    ) -> impl Future<Output = Result<DomainBlock, Self::Error>>;

    /// Replace a block's domain, severity and flags.
    ///
    /// The whole block travels, not a patch: the server would read an omitted field as
    /// "clear it". Requires `admin:write`.
    fn update_domain_block(
        &self,
        id: &str,
        draft: &DomainBlockDraft,
    ) -> impl Future<Output = Result<DomainBlock, Self::Error>>;

    /// Stop refusing a domain. Requires `admin:write`.
    fn delete_domain_block(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// Block many domains at one severity, answering what was blocked and what was not a
    /// domain at all. Requires `admin:write`.
    fn import_domain_blocks(
        &self,
        domains: &[&str],
        severity: DomainSeverity,
    ) -> impl Future<Output = Result<DomainImportOutcome, Self::Error>>;

    /// Domains allowed through a general block. Requires `admin:read`.
    fn domain_allows(
        &self,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<DomainAllow>, Self::Error>>;

    /// Carve one domain back out of a block. Requires `admin:write`.
    fn create_domain_allow(
        &self,
        domain: &str,
    ) -> impl Future<Output = Result<DomainAllow, Self::Error>>;

    /// Stop allowing a domain through. Requires `admin:write`.
    fn delete_domain_allow(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// Mail domains registrations are refused from. Requires `admin:read`.
    fn email_domain_blocks(
        &self,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<EmailDomainBlock>, Self::Error>>;

    /// Refuse registrations from a mail domain. Requires `admin:write`.
    fn create_email_domain_block(
        &self,
        domain: &str,
        allow_registration: bool,
    ) -> impl Future<Output = Result<EmailDomainBlock, Self::Error>>;

    /// Stop refusing a mail domain. Requires `admin:write`.
    fn delete_email_domain_block(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// Dashboard counts for the given keys (`new_users`, `interactions`, …) between two
    /// dates, both `YYYY-MM-DD`. Requires `admin:read`.
    fn measures(
        &self,
        keys: &[&str],
        start_at: &str,
        end_at: &str,
    ) -> impl Future<Output = Result<Vec<AdminMeasure>, Self::Error>>;
    // Relays are deliberately absent: Mastodon documents no relay endpoint
    // (`/methods/admin/relays/` is itself a 404) and `/api/v1/admin/relays` answers the
    // instance's HTML 404 page. Following a relay is a web-UI action only.
    /// A page of the audit log, narrowed by the filter. Requires `admin:read`.
    fn audit(
        &self,
        filter: &AuditFilter,
    ) -> impl Future<Output = Result<Vec<AuditEntry>, Self::Error>>;

    /// Who the grant belongs to, what it may do, and how much is waiting. Requires
    /// `admin:read`.
    fn admin_meta(&self) -> impl Future<Output = Result<AdminMeta, Self::Error>>;

    /// The flat counts behind the overview screen. Requires `admin:read`.
    fn admin_overview(&self) -> impl Future<Output = Result<AdminOverview, Self::Error>>;

    /// What the instance noticed, most recent first. Requires `admin:read`.
    fn admin_alerts(
        &self,
        unacknowledged: bool,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<AdminAlert>, Self::Error>>;

    /// Record that an alert has been seen. Requires `admin:write`.
    fn acknowledge_alert(&self, id: &str) -> impl Future<Output = Result<AdminAlert, Self::Error>>;

    /// The registered scanner services, with their rules. Requires `admin:read`.
    fn scanners(&self) -> impl Future<Output = Result<Vec<Scanner>, Self::Error>>;

    /// Register a scanner service. Requires `admin:write`.
    fn create_scanner(
        &self,
        scanner: &NewScanner,
    ) -> impl Future<Output = Result<Scanner, Self::Error>>;

    /// Replace a scanner's settings. Requires `admin:write`.
    fn update_scanner(
        &self,
        id: &str,
        scanner: &NewScanner,
    ) -> impl Future<Output = Result<Scanner, Self::Error>>;

    /// Unregister a scanner. Requires `admin:write`.
    fn delete_scanner(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// Ask a scanner what it can report and store the answer.
    ///
    /// Deliberately a button and not something a failed sweep retries: this is the only
    /// place the instance believes a service about itself, so it must be someone's
    /// decision. Requires `admin:write`.
    fn refresh_scanner(&self, id: &str) -> impl Future<Output = Result<Scanner, Self::Error>>;

    /// Decide what one label means at or above one threshold. Requires `admin:write`.
    fn set_scanner_rule(
        &self,
        scanner_id: &str,
        label: &str,
        threshold: f64,
        severity: ScanSeverity,
    ) -> impl Future<Output = Result<ScannerRule, Self::Error>>;

    /// Forget a rule, leaving the scanner's other labels alone. Requires `admin:write`.
    fn delete_scanner_rule(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// The content sets this instance holds. Requires `admin:read`.
    fn scanner_assets(&self) -> impl Future<Output = Result<Vec<ScannerAsset>, Self::Error>>;

    /// Upload a content set; the server computes the digest scanners verify against.
    /// Requires `admin:write`.
    fn upload_scanner_asset(
        &self,
        asset: &AssetUpload,
    ) -> impl Future<Output = Result<ScannerAsset, Self::Error>>;

    /// Delete a content set no scanner points at. Requires `admin:write`.
    fn delete_scanner_asset(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// The bytes of one content set, for checking what a service received.
    /// Requires `admin:read`.
    fn download_scanner_asset(
        &self,
        id: &str,
    ) -> impl Future<Output = Result<Vec<u8>, Self::Error>>;

    /// Scan results a moderator has to judge. `state` of `None` asks for all of them.
    /// Requires `admin:read`.
    fn scan_flags(
        &self,
        state: Option<FlagState>,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<ScanFlag>, Self::Error>>;

    /// Record a moderator's verdict on a flag; the scan itself stays. Requires
    /// `admin:write`.
    fn resolve_scan_flag(
        &self,
        id: &str,
        state: FlagState,
    ) -> impl Future<Output = Result<ScanFlag, Self::Error>>;

    /// Scans waiting, running or finished. `state` is one of `pending`, `running`, `done`
    /// or `failed`, or `None` for every state. Requires `admin:read`.
    fn scan_queue(
        &self,
        state: Option<&str>,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<Scan>, Self::Error>>;

    /// Queue a subject by hand, answering how many scanners picked it up. Requires
    /// `admin:write`.
    fn enqueue_scan(&self, request: &ScanRequest)
    -> impl Future<Output = Result<u64, Self::Error>>;

    /// Warnings a moderator can reuse. Requires `admin:read`.
    fn warning_presets(&self) -> impl Future<Output = Result<Vec<WarningPreset>, Self::Error>>;

    /// Save a new warning preset. Requires `admin:write`.
    fn create_warning_preset(
        &self,
        title: &str,
        text: &str,
    ) -> impl Future<Output = Result<WarningPreset, Self::Error>>;

    /// Replace a preset's title and text. Requires `admin:write`.
    fn update_warning_preset(
        &self,
        id: &str,
        title: &str,
        text: &str,
    ) -> impl Future<Output = Result<WarningPreset, Self::Error>>;

    /// Delete a preset. Requires `admin:write`.
    fn delete_warning_preset(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// Every announcement, published or not, for staff. Requires `admin:read`.
    fn admin_announcements(&self) -> impl Future<Output = Result<Vec<Announcement>, Self::Error>>;

    /// Publish a new announcement. Requires `admin:write`.
    fn create_announcement(
        &self,
        draft: AnnouncementDraft,
    ) -> impl Future<Output = Result<Announcement, Self::Error>>;

    /// Replace an announcement's fields. Requires `admin:write`.
    fn update_announcement(
        &self,
        id: &str,
        draft: AnnouncementDraft,
    ) -> impl Future<Output = Result<Announcement, Self::Error>>;

    /// Withdraw an announcement. Requires `admin:write`.
    fn delete_announcement(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// Every process this instance knows about, and how it is doing right now.
    ///
    /// The answer is assembled from live calls bounded at a quarter of a second, so a node
    /// that is down lengthens the answer by that budget rather than by a timeout.
    /// Requires `admin:read`.
    fn topology(&self) -> impl Future<Output = Result<Topology, Self::Error>>;

    /// The last entries one node kept in memory, oldest first.
    ///
    /// Nothing is stored: a restart of that node empties this. Requires `admin:read`.
    fn node_logs(
        &self,
        name: &str,
        min_level: &str,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<NodeLog>, Self::Error>>;

    /// Asks one process to do its pass now instead of on its next tick (`admin:write`).
    ///
    /// `work` names the pass, and a process accepts only the name it does: `scan`, `prune`
    /// or `calc`. The durable path to a worker is still its queue.
    fn trigger_node(
        &self,
        name: &str,
        work: &str,
    ) -> impl Future<Output = Result<TriggerOutcome, Self::Error>>;
}

/// What the account asked for, and what the instance wants said.
///
/// Preferences belong to the account rather than to the browser: a reader who always
/// posts unlisted should not have to set that in every client they sign into.
pub trait SettingsActions {
    /// Error type of the transport.
    type Error: Debug;

    /// The account's posting and reading defaults.
    fn preferences(&self) -> impl Future<Output = Result<Preferences, Self::Error>>;

    /// Announcements the instance is showing.
    fn announcements(&self) -> impl Future<Output = Result<Vec<Announcement>, Self::Error>>;

    /// Marks an announcement as read, so it stops appearing.
    fn dismiss_announcement(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;
}

/// The approval queue: who may join this instance.
///
/// Invites are deliberately absent. Mastodon serves them only to its own web UI —
/// `GET /api/v1/invites` answers 404 on every version we checked (4.3.8, mastodon.social,
/// fosstodon.org, chaos.social) — so a client cannot hand them out, and a method that
/// always failed would be worse than none. The screen links to the instance's own page.
pub trait RegistrationActions {
    type Error: Debug;

    /// Accounts waiting for a moderator's decision.
    fn pending_accounts(
        &self,
        limit: u16,
    ) -> impl Future<Output = Result<Vec<AdminAccount>, Self::Error>>;
    fn approve_account(&self, id: &str) -> impl Future<Output = Result<AdminAccount, Self::Error>>;
    fn reject_account(&self, id: &str) -> impl Future<Output = Result<AdminAccount, Self::Error>>;
}

/// What a server says about itself.
///
/// Readable without a token: a name, a version and a contact address are what someone
/// deciding whether to sign in needs first.
pub trait InstanceMeta {
    type Error: Debug;

    fn instance_info(&self) -> impl Future<Output = Result<InstanceInfo, Self::Error>>;

    /// The emoji this instance offers in its picker, federated ones included.
    ///
    /// Mastodon calls these "custom emojis". The list is the instance's cached set, so it
    /// already contains every emoji it has seen from other servers — which is exactly what
    /// a composer needs: an emoji offered here is one the instance will render, while an
    /// emoji copied from elsewhere may not be in the set at all.
    fn custom_emojis(&self) -> impl Future<Output = Result<Vec<CustomEmoji>, Self::Error>>;

    /// The languages this instance publishes posts in.
    fn languages(&self) -> impl Future<Output = Result<Vec<Language>, Self::Error>>;

    /// Tags this instance is promoting, busiest first.
    ///
    /// Readable without a token: it is what a visitor sees on the front page, and it is
    /// what a reader's own rail shows when they ask for "what is happening here".
    fn trending_tags(&self, limit: usize) -> impl Future<Output = Result<Vec<Tag>, Self::Error>>;

    /// Posts this instance is promoting, in the instance's order.
    ///
    /// The ranking is the instance's, not this client's: a reader who wanted our own idea
    /// of interesting would be reading the recommender tab instead.
    fn trending_statuses(
        &self,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<Status>, Self::Error>>;

    /// Links this instance is promoting, busiest first.
    ///
    /// Same ranking as the trending tags and statuses, for the front page's third rail.
    fn trending_links(
        &self,
        limit: usize,
    ) -> impl Future<Output = Result<Vec<TrendingLink>, Self::Error>>;
}

/// Featured tags, and the instance's rules.
///
/// The rules are read without a token: they are what someone about to join is asked to
/// accept, so requiring a session to see them would be backwards.
pub trait HighlightActions {
    /// Error type of the transport.
    type Error: Debug;

    /// The tags an account has put on its profile.
    fn featured_tags(
        &self,
        id: &LocalId,
    ) -> impl Future<Output = Result<Vec<FeaturedTag>, Self::Error>>;

    /// Features a tag on the authenticated account's profile.
    fn feature_tag(&self, name: &str) -> impl Future<Output = Result<FeaturedTag, Self::Error>>;

    /// Removes a tag from the profile.
    fn unfeature_tag(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;

    /// The instance's code of conduct.
    fn rules(&self) -> impl Future<Output = Result<Vec<Rule>, Self::Error>>;
}

/// Tags: the account's featured set, one tag's metadata, following a tag.
///
/// Following a tag is not the same as starring a post: it puts every post with that tag
/// on the home timeline, which is how a reader subscribes to a topic rather than a person.
pub trait TagActions {
    /// Error type of the transport.
    type Error: Debug;

    /// The featured tags of the signed-in account.
    ///
    /// `HighlightActions::featured_tags` reads anyone's, which is what a profile page
    /// needs; this one reads the account's own list, which is what a settings screen edits.
    fn own_featured_tags(&self) -> impl Future<Output = Result<Vec<FeaturedTag>, Self::Error>>;

    /// One tag's metadata: how many people follow it, how busy it is.
    fn tag(&self, name: &str) -> impl Future<Output = Result<Tag, Self::Error>>;

    /// Follows a tag, so its posts enter the home timeline.
    fn follow_tag(&self, name: &str) -> impl Future<Output = Result<Tag, Self::Error>>;

    /// Unfollows a tag.
    fn unfollow_tag(&self, name: &str) -> impl Future<Output = Result<Tag, Self::Error>>;

    /// Tags the instance suggests featuring, busiest first.
    ///
    /// Answered as `FeaturedTag` documents because that is what Mastodon sends and every
    /// client parses; a profile editor offers them as one-click additions so the reader
    /// does not have to remember a name.
    fn featured_tag_suggestions(
        &self,
    ) -> impl Future<Output = Result<Vec<FeaturedTag>, Self::Error>>;
}

/// Posts the instance will publish later.
///
/// Scheduling is the same call as posting with a time attached, which is why it takes a
/// [`Draft`] rather than its own shape: the content is identical, only the moment differs.
pub trait ScheduleActions {
    /// Error type of the transport.
    type Error: Debug;

    /// Everything waiting to be published, soonest first.
    fn scheduled(&self) -> impl Future<Output = Result<Vec<ScheduledStatus>, Self::Error>>;

    /// Queues a draft for `at` (RFC 3339).
    fn schedule(
        &self,
        draft: &Draft,
        at: &str,
    ) -> impl Future<Output = Result<ScheduledStatus, Self::Error>>;

    /// Cancels a queued post.
    fn cancel_scheduled(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;
}
