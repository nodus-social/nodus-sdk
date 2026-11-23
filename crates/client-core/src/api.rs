//! The instance surface, as a trait.
//!
//! Everything the client can ask a Mastodon instance to do is declared here once,
//! as [`InstanceApi`]. Implementations are the thing that differs between
//! targets: a `gloo-net` implementation in the browser, a `reqwest` one in the
//! sync worker, and a scripted one in tests. Consumers are generic over the
//! trait, so the same SDK state machine runs in all three.
//!
//! The methods return `impl Future` rather than being `async fn` in the trait
//! declaration: it keeps the returned future nameable for implementors and avoids
//! the `async_fn_in_trait` warning on a public API.

use core::fmt::Debug;
use core::future::Future;

use crate::model::{
    Account, Announcement, Context, Conversation, Draft, FeaturedTag, Filter, List, LocalId,
    MarkerTimeline, Markers, Notification, Page, PageParams, Preferences, Relationship,
    SearchResults, Status, StreamKind,
};

/// The read and write operations the client performs against one instance.
///
/// `Error` is the transport's own error type, so an implementation never has to
/// translate into a shared enum it does not own; the SDK maps it into
/// [`crate::Error`] at its boundary.
pub trait InstanceApi {
    /// Transport error type.
    type Error: Debug;

    /// The signed-in account, used to validate a token and to render the profile.
    fn verify_credentials(&self) -> impl Future<Output = Result<Account, Self::Error>>;

    /// The signed-in account with the preferences only it can read.
    ///
    /// Same endpoint as [`InstanceApi::verify_credentials`], but keeps the `source` object —
    /// which is where the writer's default visibility and posting language live. A picker
    /// that guessed those from the instance would be wrong for exactly the writers who
    /// configured them.
    fn credential_account(
        &self,
    ) -> impl Future<Output = Result<crate::model::CredentialAccount, Self::Error>>;

    /// A timeline, newest first.
    fn timeline(
        &self,
        kind: StreamKind,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Status>, Self::Error>>;

    /// A hashtag timeline, newest first.
    fn tag_timeline(
        &self,
        tag: &str,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Status>, Self::Error>>;

    /// A list's timeline, newest first.
    fn list_timeline(
        &self,
        list_id: &str,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Status>, Self::Error>>;

    /// One status.
    fn status(&self, id: &str) -> impl Future<Output = Result<Status, Self::Error>>;

    /// The replies above and below a status.
    fn status_context(&self, id: &str) -> impl Future<Output = Result<Context, Self::Error>>;

    /// Notifications, newest first.
    fn notifications(
        &self,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Notification>, Self::Error>>;

    /// One account.
    fn account(&self, id: &LocalId) -> impl Future<Output = Result<Account, Self::Error>>;

    /// Resolve a local or remote `acct` on the instance that owns the token.
    ///
    /// The lookup endpoint is public on Mastodon; it is how an origin-first crawler
    /// turns `user@host` into the account id that host uses in its own API.
    fn account_lookup(&self, acct: &str) -> impl Future<Output = Result<Account, Self::Error>>;

    /// An account's followers, newest first.
    fn account_followers(
        &self,
        id: &LocalId,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Account>, Self::Error>>;

    /// The accounts an account follows, newest first.
    fn account_following(
        &self,
        id: &LocalId,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Account>, Self::Error>>;

    /// An account's statuses.
    fn account_statuses(
        &self,
        id: &LocalId,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Status>, Self::Error>>;

    /// The tags an account has pinned to its profile.
    ///
    /// Public on Mastodon, so a profile page can paint it for any account the
    /// instance knows; the settings screen asks for the signed-in account's own id.
    fn featured_tags(
        &self,
        id: &LocalId,
    ) -> impl Future<Output = Result<Vec<FeaturedTag>, Self::Error>>;

    /// Statuses the account bookmarked, newest first.
    fn bookmarks(
        &self,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Status>, Self::Error>>;

    /// Statuses the account favourited, newest first.
    fn favourites(
        &self,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Status>, Self::Error>>;

    /// Direct-message threads, newest first.
    fn conversations(
        &self,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Conversation>, Self::Error>>;

    /// The account's lists, whole rather than paged.
    fn lists(&self) -> impl Future<Output = Result<Vec<List>, Self::Error>>;

    /// Accounts waiting for this account to approve them.
    ///
    /// The endpoint answers a bare array: follow requests are not paged by id, so
    /// there is no `Page` to advance.
    fn follow_requests(&self) -> impl Future<Output = Result<Vec<Account>, Self::Error>>;

    /// Accounts this account mutes, newest first.
    fn mutes(&self, params: PageParams)
    -> impl Future<Output = Result<Page<Account>, Self::Error>>;

    /// Accounts this account blocks, newest first.
    fn blocks(
        &self,
        params: PageParams,
    ) -> impl Future<Output = Result<Page<Account>, Self::Error>>;

    /// The signed-in account's posting and reading defaults.
    ///
    /// The shell gates this behind `Feature::Preferences`; this trait has no capability
    /// handle, so the transport itself cannot enforce that here.
    fn preferences(&self) -> impl Future<Output = Result<Preferences, Self::Error>>;

    /// The filters this account keeps.
    ///
    /// The server applies them to timelines on its own; this read is for the
    /// settings list, and for matching a thread assembled from cached statuses.
    fn filters(&self) -> impl Future<Output = Result<Vec<Filter>, Self::Error>>;

    /// The server-held reading positions for `timelines`.
    fn markers(
        &self,
        timelines: &[MarkerTimeline],
    ) -> impl Future<Output = Result<Markers, Self::Error>>;

    /// Announcements the instance is showing.
    fn announcements(&self) -> impl Future<Output = Result<Vec<Announcement>, Self::Error>>;

    /// One page of the instance's account directory.
    ///
    /// Paged by offset rather than by id: the endpoint answers a bare array, so a
    /// caller advances by the number of accounts it received.
    fn directory(
        &self,
        local_only: bool,
        offset: u32,
    ) -> impl Future<Output = Result<Vec<Account>, Self::Error>>;

    /// Searches accounts, statuses and hashtags.
    ///
    /// `resolve` is the difference between what this instance already knows and
    /// going out to fetch an unknown `acct`, which can take seconds.
    fn search(
        &self,
        query: &str,
        resolve: bool,
    ) -> impl Future<Output = Result<SearchResults, Self::Error>>;

    /// Whether the signed-in account follows `id`.
    fn relationship(&self, id: &LocalId)
    -> impl Future<Output = Result<Relationship, Self::Error>>;

    /// Follows `id`, returning the resulting relationship.
    fn follow(&self, id: &LocalId) -> impl Future<Output = Result<Relationship, Self::Error>>;

    /// Unfollows `id`, returning the resulting relationship.
    fn unfollow(&self, id: &LocalId) -> impl Future<Output = Result<Relationship, Self::Error>>;

    /// Publishes a draft.
    fn post_status(&self, draft: &Draft) -> impl Future<Output = Result<Status, Self::Error>>;

    /// Deletes a status the account owns.
    fn delete_status(&self, id: &str) -> impl Future<Output = Result<(), Self::Error>>;
}

/// Implemented by transports that can hold a streaming socket.
///
/// Separate from [`InstanceApi`] because streaming is optional: the sync worker
/// polls, the browser streams, and tests do neither.
pub trait Streaming {
    /// Error reported by the socket.
    type Error: Debug;

    /// Opens a socket for `kind`, delivering raw JSON event payloads.
    ///
    /// The implementation owns reconnection; the SDK only observes the resulting
    /// `StreamState` transitions in `nodus-state`.
    fn subscribe(&self, kind: StreamKind) -> impl Future<Output = Result<(), Self::Error>>;
}
