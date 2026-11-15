//! Sync consent is state like any other: a command, not a view-local flag.

use nodus_client_core::model::{
    Account, Context, Draft, LocalId, Notification, Page, PageParams, Relationship, Status,
};
use nodus_client_core::{InstanceApi, StreamKind};
use nodus_shared::{AccountFeed, EventBatch, MeResponse, Meta, PostFeed};
use nodus_state::{Command, Discovery, SyncConsent, TimelineRequest};

use nodus_persist::MemoryPersistence;

use crate::clock::{FixedClock, SystemClock};
use crate::sdk::Sdk;
use crate::shell::{Unavailable, UnavailableBackend};

/// Answers `me` with a fixed consent and fails everything else.
struct Consented(SyncConsent);

/// A finished failure: the stub answers on the first poll, like the shell's.
fn fail<T>() -> core::future::Ready<Result<T, Unavailable>> {
    core::future::ready(Err(Unavailable))
}

impl Discovery for Consented {
    type Error = Unavailable;

    fn meta(&self) -> impl core::future::Future<Output = Result<Meta, Self::Error>> {
        fail::<Meta>()
    }

    fn me(&self) -> impl core::future::Future<Output = Result<MeResponse, Self::Error>> {
        let consent = self.0;
        core::future::ready(Ok(MeResponse {
            acct: "someone@remote.test".to_owned(),
            instance_host: "remote.test".to_owned(),
            sync_enabled: consent.enabled,
            telemetry_opt_in: consent.telemetry_opt_in,
            contract: nodus_shared::CONTRACT_VERSION.to_owned(),
        }))
    }

    fn feed(
        &self,
        _cursor: Option<&str>,
    ) -> impl core::future::Future<Output = Result<AccountFeed, Self::Error>> {
        fail::<AccountFeed>()
    }

    fn posts(
        &self,
        _cursor: Option<&str>,
    ) -> impl core::future::Future<Output = Result<PostFeed, Self::Error>> {
        fail::<PostFeed>()
    }

    fn refresh(
        &self,
        _feed_id: &str,
    ) -> impl core::future::Future<Output = Result<AccountFeed, Self::Error>> {
        fail::<AccountFeed>()
    }

    fn send_events(
        &self,
        _batch: &EventBatch,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }

    fn set_sync(
        &self,
        _enable: bool,
        _telemetry_opt_in: bool,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }
}

fn sdk_with(
    consent: SyncConsent,
) -> Sdk<UnavailableBackend, Consented, MemoryPersistence, SystemClock> {
    Sdk::new(
        UnavailableBackend,
        Consented(consent),
        MemoryPersistence::new(),
        SystemClock,
    )
}

/// Polls once: every future here is ready on the first poll.
fn block_on<F: core::future::Future>(future: F) -> F::Output {
    use core::task::{Context, Poll, Waker};
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("the port under test never pends"),
    }
}

#[test]
fn the_server_is_what_answers_whether_sync_is_on() {
    let sdk = sdk_with(SyncConsent {
        enabled: true,
        telemetry_opt_in: false,
    });
    assert!(
        sdk.state().sync.get().is_none(),
        "nothing is known before the server answers"
    );

    let consent = block_on(sdk.load_sync_consent()).expect("me answers");

    assert!(consent.enabled);
    assert_eq!(
        sdk.state().sync.get().map(|consent| consent.enabled),
        Some(true),
        "the answer is state, not a local flag"
    );
}

#[test]
fn signing_out_forgets_the_consent() {
    let sdk = sdk_with(SyncConsent {
        enabled: true,
        telemetry_opt_in: true,
    });
    block_on(sdk.load_sync_consent()).expect("me answers");

    sdk.dispatch(Command::SignedOut);

    assert_eq!(sdk.state().sync.get(), None);
}

/// Serves prepared pages of statuses, and counts how many times it was asked.
struct Pages {
    calls: core::cell::Cell<usize>,
    pages: Vec<Vec<Status>>,
}

/// A status with only the fields the timeline logic reads.
fn status(id: &str) -> Status {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "created_at": "2026-11-07T00:00:00.000Z",
        "content": "<p>hello</p>",
        "visibility": "public",
        "account": {
            "id": format!("account-{id}"),
            "acct": "someone@remote.test",
            "display_name": "Someone"
        }
    }))
    .expect("a status the model accepts")
}

impl Pages {
    fn of(pages: Vec<Vec<&str>>) -> Self {
        Self {
            calls: core::cell::Cell::new(0),
            pages: pages
                .into_iter()
                .map(|ids| ids.into_iter().map(status).collect())
                .collect(),
        }
    }
}

impl InstanceApi for Pages {
    fn credential_account(
        &self,
    ) -> impl core::future::Future<
        Output = Result<nodus_client_core::CredentialAccount, <Self as InstanceApi>::Error>,
    > {
        core::future::ready(Err(Unavailable))
    }

    type Error = Unavailable;

    fn verify_credentials(
        &self,
    ) -> impl core::future::Future<Output = Result<Account, Self::Error>> {
        fail::<Account>()
    }

    fn timeline(
        &self,
        _kind: StreamKind,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        let index = self.calls.get();
        self.calls.set(index + 1);
        let items = self.pages.get(index).cloned().unwrap_or_default();
        let next = items.last().map(|status| status.id.clone());
        core::future::ready(Ok(Page::new(items).with_next(next)))
    }

    fn tag_timeline(
        &self,
        _tag: &str,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        fail::<Page<Status>>()
    }

    fn list_timeline(
        &self,
        _list_id: &str,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        fail::<Page<Status>>()
    }

    fn status(&self, _id: &str) -> impl core::future::Future<Output = Result<Status, Self::Error>> {
        fail::<Status>()
    }

    fn status_context(
        &self,
        _id: &str,
    ) -> impl core::future::Future<Output = Result<Context, Self::Error>> {
        fail::<Context>()
    }

    fn notifications(
        &self,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Notification>, Self::Error>> {
        fail::<Page<Notification>>()
    }

    fn account(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Account, Self::Error>> {
        fail::<Account>()
    }

    fn account_lookup(
        &self,
        _acct: &str,
    ) -> impl core::future::Future<Output = Result<Account, Self::Error>> {
        fail::<Account>()
    }

    fn account_followers(
        &self,
        _id: &LocalId,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Account>, Self::Error>> {
        fail::<Page<Account>>()
    }

    fn account_following(
        &self,
        _id: &LocalId,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Account>, Self::Error>> {
        fail::<Page<Account>>()
    }

    fn account_statuses(
        &self,
        _id: &LocalId,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        fail::<Page<Status>>()
    }

    fn relationship(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn follow(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn unfollow(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn post_status(
        &self,
        _draft: &Draft,
    ) -> impl core::future::Future<Output = Result<Status, Self::Error>> {
        fail::<Status>()
    }

    fn delete_status(
        &self,
        _id: &str,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }
}

/// One request, one page, and the page is readable without another request.
#[test]
fn a_loaded_page_is_readable_without_asking_again() {
    let instance = Pages::of(vec![vec!["1", "2"]]);
    instance.calls.set(0);
    let sdk = Sdk::new(
        instance,
        Consented(SyncConsent::default()),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );

    let request = TimelineRequest::Home;
    block_on(sdk.ensure_timeline(request.clone(), None, 20, 60_000)).expect("a page arrives");

    let (statuses, cursor) = sdk.read_timeline(&request);
    assert_eq!(
        statuses
            .iter()
            .map(|status| status.id.clone())
            .collect::<Vec<_>>(),
        vec!["1".to_owned(), "2".to_owned()],
    );
    assert_eq!(cursor.as_deref(), Some("2"));
}

/// Inside its TTL a held page is not fetched again.
#[test]
fn ensure_timeline_trusts_a_page_inside_its_ttl() {
    let instance = Pages::of(vec![vec!["1"], vec!["9"]]);
    let sdk = Sdk::new(
        instance,
        Consented(SyncConsent::default()),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );

    let request = TimelineRequest::Home;
    block_on(sdk.ensure_timeline(request.clone(), None, 20, 60_000)).expect("first page");
    let second = block_on(sdk.ensure_timeline(request.clone(), None, 20, 60_000)).expect("read");

    assert!(second.is_none(), "nothing was stale, so nothing was asked");
    assert_eq!(sdk.read_timeline(&request).0.len(), 1);
}

/// A cursor adds to what is held instead of replacing it.
#[test]
fn an_older_page_is_appended_not_substituted() {
    let instance = Pages::of(vec![vec!["1", "2"], vec!["3"]]);
    let sdk = Sdk::new(
        instance,
        Consented(SyncConsent::default()),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );

    let request = TimelineRequest::Home;
    block_on(sdk.load_timeline(request.clone(), None, 20)).expect("newest page");
    block_on(sdk.load_timeline(request.clone(), Some("2"), 20)).expect("older page");

    let ids: Vec<String> = sdk
        .read_timeline(&request)
        .0
        .into_iter()
        .map(|status| status.id)
        .collect();
    assert_eq!(ids, vec!["1".to_owned(), "2".to_owned(), "3".to_owned()]);
}

/// An `update` frame makes its status readable without any request.
#[test]
fn an_update_event_lands_in_the_entity_store() {
    let sdk = Sdk::new(
        Pages::of(vec![]),
        Consented(SyncConsent::default()),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );
    let frame = serde_json::json!({
        "event": "update",
        "payload": serde_json::to_string(&status("7")).expect("a status serialises"),
    })
    .to_string();

    assert!(sdk.apply_frame(&frame), "an update changes state");
    assert!(
        sdk.read_status("7").is_some(),
        "the status is held by id, so any page can render it"
    );
}

/// A `delete` frame removes the status and its id from every held page.
#[test]
fn a_delete_event_leaves_no_hole_in_a_page() {
    let instance = Pages::of(vec![vec!["1", "2"]]);
    let sdk = Sdk::new(
        instance,
        Consented(SyncConsent::default()),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );
    let request = TimelineRequest::Home;
    block_on(sdk.ensure_timeline(request.clone(), None, 20, 60_000)).expect("a page arrives");

    let frame = serde_json::json!({"event": "delete", "payload": "1"}).to_string();
    assert!(
        sdk.apply_frame(&frame),
        "a delete of a held status changes state"
    );

    let (statuses, _) = sdk.read_timeline(&request);
    assert_eq!(
        statuses
            .iter()
            .map(|status| status.id.clone())
            .collect::<Vec<_>>(),
        vec!["2".to_owned()],
        "the deleted id leaves the page rather than rendering as a hole",
    );
}

fn sample_dm_conversation() -> (Status, nodus_client_core::model::Conversation) {
    let dm_status: Status = serde_json::from_value(serde_json::json!({
        "id": "dm-1",
        "created_at": "2026-11-07T00:00:00.000Z",
        "content": "<p>secret dm</p>",
        "visibility": "direct",
        "account": {
            "id": "acct-dm",
            "acct": "alice@remote.test",
            "display_name": "Alice"
        }
    }))
    .expect("dm status");
    let conversation = serde_json::from_value(serde_json::json!({
        "id": "conv-1",
        "unread": true,
        "accounts": [dm_status.account.clone()],
        "last_status": dm_status.clone()
    }))
    .expect("conversation");
    (dm_status, conversation)
}

fn sample_feeds() -> (AccountFeed, PostFeed) {
    let account_feed = serde_json::from_value(serde_json::json!({
        "feed_id": "af-1",
        "generated_at": "2026-11-07T00:00:00Z",
        "contract": "0.1.0",
        "model_version": 1,
        "similarity_age_hours": 1.0,
        "items": [{
            "item_id": 10,
            "account": {
                "id": "bob-1",
                "acct": "bob@remote.test",
                "display_name": "Bob",
                "avatar_url": ""
            },
            "reason": { "code": "followed_by_similar_users" }
        }]
    }))
    .expect("account feed");
    let post_feed = serde_json::from_value(serde_json::json!({
        "feed_id": "pf-1",
        "generated_at": "2026-11-07T00:00:00Z",
        "contract": "0.1.0",
        "model_version": 1,
        "items": [{
            "item_id": 20,
            "status_id": "p-20",
            "status_remote_id": "p-20",
            "account_acct": "bob@remote.test",
            "account_remote_id": "bob-1",
            "account_display_name": "Bob",
            "account_avatar_url": "",
            "text": "<p>recommended</p>",
            "published_at": "2026-11-07T00:00:00Z",
            "favourites_count": 0,
            "reblogs_count": 0,
            "replies_count": 0,
            "instance_host": "remote.test",
            "reason": { "code": "followed_by_similar_users" }
        }]
    }))
    .expect("post feed");
    (account_feed, post_feed)
}

#[test]
fn signing_out_clears_all_private_state() {
    use nodus_protocol::Capabilities;
    use nodus_state::page::{PageItems, PageKind};
    use nodus_state::{Session, StreamState, Timeline};

    let sdk = sdk_with(SyncConsent {
        enabled: true,
        telemetry_opt_in: true,
    });
    block_on(sdk.load_sync_consent()).expect("me answers");
    sdk.adopt_session(Session {
        host: "remote.test".to_owned(),
        account_id: "self-1".to_owned(),
        acct: "someone@remote.test".to_owned(),
        scopes: vec!["read".to_owned(), "write".to_owned()],
    });
    sdk.dispatch(Command::CapabilitiesLoaded(Capabilities::from_parts(
        "0.2.0",
        &["batch".to_owned(), "events".to_owned()],
    )));
    sdk.dispatch(Command::StreamChanged(StreamState::Connected));

    let (dm_status, conversation) = sample_dm_conversation();
    sdk.dispatch(Command::AccountsUpserted {
        accounts: vec![dm_status.account.clone()],
    });
    sdk.dispatch(Command::StatusesUpserted {
        statuses: vec![dm_status],
    });
    sdk.adopt_items(
        PageKind::Conversations,
        PageItems::Conversations(vec![conversation]),
        Some("conv-cursor".to_owned()),
    );
    sdk.adopt_page(
        Timeline::Home,
        None,
        vec![status("home-1")],
        Some("home-cursor".to_owned()),
    );

    let (account_feed, post_feed) = sample_feeds();
    sdk.dispatch(Command::FeedLoaded(account_feed));
    sdk.dispatch(Command::PostsLoaded {
        feed: post_feed,
        append: false,
    });

    block_on(sdk.sign_out()).expect("sign out succeeds");

    let state = sdk.state();
    assert_eq!(state.capabilities.get(), Capabilities::mastodon());
    assert!(state.session.get().is_none());
    assert!(state.sync.get().is_none());
    assert_eq!(state.stream.get(), StreamState::Disconnected);
    assert_eq!(state.feed.get().items, Vec::new());
    assert_eq!(state.posts.get().items, Vec::new());
    assert!(state.posts.get().feed_id.is_none());
    assert!(state.timelines.with(std::collections::HashMap::is_empty));
    assert!(sdk.read_status("dm-1").is_none());
    assert!(state.statuses.with(std::collections::HashMap::is_empty));
    assert!(
        state
            .pages
            .with(|pages| pages.get(&PageKind::Conversations).is_none())
    );
    assert!(state.pages.with(std::collections::HashMap::is_empty));
    assert!(sdk.read_account("acct-dm").is_none());
    assert!(state.accounts.with(std::collections::HashMap::is_empty));
}

fn account(id: &str) -> Account {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "acct": format!("{id}@remote.test"),
        "display_name": id
    }))
    .expect("valid account")
}

fn boosted_status(id: &str, booster: &str, original_id: &str, author: &str) -> Status {
    let mut inner = status(original_id);
    inner.account = account(author);
    let mut outer = status(id);
    outer.account = account(booster);
    outer.reblog = Some(Box::new(inner));
    outer
}

/// Mock instance that distinguishes home, list, and browse endpoints.
struct RichInstance {
    timeline_calls: core::cell::Cell<usize>,
    list_calls: core::cell::RefCell<Vec<String>>,
    list_statuses: Vec<Status>,
    favourites_statuses: Vec<Status>,
    notifications: Vec<Notification>,
    conversations: Vec<nodus_client_core::model::Conversation>,
    search_results: nodus_client_core::model::SearchResults,
}

impl Default for RichInstance {
    fn default() -> Self {
        Self {
            timeline_calls: core::cell::Cell::new(0),
            list_calls: core::cell::RefCell::new(Vec::new()),
            list_statuses: Vec::new(),
            favourites_statuses: Vec::new(),
            notifications: Vec::new(),
            conversations: Vec::new(),
            search_results: nodus_client_core::model::SearchResults::default(),
        }
    }
}

struct FeedProbe {
    calls: core::cell::Cell<usize>,
    fail: core::cell::Cell<bool>,
}

impl FeedProbe {
    fn new(fail: bool) -> Self {
        Self {
            calls: core::cell::Cell::new(0),
            fail: core::cell::Cell::new(fail),
        }
    }
}

impl InstanceApi for RichInstance {
    type Error = Unavailable;

    fn verify_credentials(
        &self,
    ) -> impl core::future::Future<Output = Result<Account, Self::Error>> {
        fail::<Account>()
    }

    fn credential_account(
        &self,
    ) -> impl core::future::Future<Output = Result<nodus_client_core::CredentialAccount, Self::Error>>
    {
        fail::<nodus_client_core::CredentialAccount>()
    }

    fn timeline(
        &self,
        _kind: StreamKind,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        self.timeline_calls.set(self.timeline_calls.get() + 1);
        core::future::ready(Ok(Page::new(vec![status("home-only")])))
    }

    fn tag_timeline(
        &self,
        _tag: &str,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        fail::<Page<Status>>()
    }

    fn list_timeline(
        &self,
        list_id: &str,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        self.list_calls.borrow_mut().push(list_id.to_owned());
        core::future::ready(Ok(Page::new(self.list_statuses.clone())))
    }

    fn status(&self, _id: &str) -> impl core::future::Future<Output = Result<Status, Self::Error>> {
        fail::<Status>()
    }

    fn status_context(
        &self,
        _id: &str,
    ) -> impl core::future::Future<Output = Result<Context, Self::Error>> {
        fail::<Context>()
    }

    fn notifications(
        &self,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Notification>, Self::Error>> {
        core::future::ready(Ok(Page::new(self.notifications.clone())))
    }

    fn account(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Account, Self::Error>> {
        fail::<Account>()
    }

    fn account_lookup(
        &self,
        _acct: &str,
    ) -> impl core::future::Future<Output = Result<Account, Self::Error>> {
        fail::<Account>()
    }

    fn account_followers(
        &self,
        _id: &LocalId,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Account>, Self::Error>> {
        fail::<Page<Account>>()
    }

    fn account_following(
        &self,
        _id: &LocalId,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Account>, Self::Error>> {
        fail::<Page<Account>>()
    }

    fn account_statuses(
        &self,
        _id: &LocalId,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        fail::<Page<Status>>()
    }

    fn relationship(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn follow(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn unfollow(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn post_status(
        &self,
        _draft: &Draft,
    ) -> impl core::future::Future<Output = Result<Status, Self::Error>> {
        fail::<Status>()
    }

    fn delete_status(
        &self,
        _id: &str,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }
}

impl nodus_client_core::actions::Browse for RichInstance {
    type Error = Unavailable;

    fn favourites(
        &self,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        core::future::ready(Ok(Page::new(self.favourites_statuses.clone())))
    }

    fn bookmarks(
        &self,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        fail::<Page<Status>>()
    }

    fn conversations(
        &self,
        _params: PageParams,
    ) -> impl core::future::Future<
        Output = Result<Page<nodus_client_core::model::Conversation>, Self::Error>,
    > {
        core::future::ready(Ok(Page::new(self.conversations.clone())))
    }

    fn mark_conversation_read(
        &self,
        _id: &str,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }

    fn directory(
        &self,
        _local_only: bool,
        _offset: u32,
    ) -> impl core::future::Future<Output = Result<Vec<Account>, Self::Error>> {
        fail::<Vec<Account>>()
    }

    fn mute_conversation(
        &self,
        _id: &str,
        _muted: bool,
    ) -> impl core::future::Future<Output = Result<nodus_client_core::model::Conversation, Self::Error>>
    {
        fail::<nodus_client_core::model::Conversation>()
    }

    fn lists(
        &self,
    ) -> impl core::future::Future<Output = Result<Vec<nodus_client_core::model::List>, Self::Error>>
    {
        fail::<Vec<nodus_client_core::model::List>>()
    }

    fn create_list(
        &self,
        _title: &str,
    ) -> impl core::future::Future<Output = Result<nodus_client_core::model::List, Self::Error>>
    {
        fail::<nodus_client_core::model::List>()
    }

    fn rename_list(
        &self,
        _id: &str,
        _title: &str,
    ) -> impl core::future::Future<Output = Result<nodus_client_core::model::List, Self::Error>>
    {
        fail::<nodus_client_core::model::List>()
    }

    fn delete_list(
        &self,
        _id: &str,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }

    fn add_to_list(
        &self,
        _list_id: &str,
        _accounts: &[LocalId],
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }

    fn remove_from_list(
        &self,
        _list_id: &str,
        _accounts: &[LocalId],
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }

    fn list_accounts(
        &self,
        _list_id: &str,
    ) -> impl core::future::Future<Output = Result<Vec<Account>, Self::Error>> {
        fail::<Vec<Account>>()
    }

    fn list_timeline(
        &self,
        list_id: &str,
        params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Status>, Self::Error>> {
        <Self as InstanceApi>::list_timeline(self, list_id, params)
    }
}

impl nodus_client_core::actions::AccountActions for RichInstance {
    type Error = Unavailable;

    fn mute(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn unmute(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn block(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn unblock(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn relationships(
        &self,
        _ids: &[LocalId],
    ) -> impl core::future::Future<Output = Result<Vec<Relationship>, Self::Error>> {
        fail::<Vec<Relationship>>()
    }

    fn followers(
        &self,
        _id: &LocalId,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Account>, Self::Error>> {
        fail::<Page<Account>>()
    }

    fn following(
        &self,
        _id: &LocalId,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Account>, Self::Error>> {
        fail::<Page<Account>>()
    }

    fn mutes(
        &self,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Account>, Self::Error>> {
        fail::<Page<Account>>()
    }

    fn blocks(
        &self,
        _params: PageParams,
    ) -> impl core::future::Future<Output = Result<Page<Account>, Self::Error>> {
        fail::<Page<Account>>()
    }

    fn follow_requests(
        &self,
    ) -> impl core::future::Future<Output = Result<Vec<Account>, Self::Error>> {
        fail::<Vec<Account>>()
    }

    fn authorize_follow_request(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<Relationship, Self::Error>> {
        fail::<Relationship>()
    }

    fn reject_follow_request(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }

    fn update_profile(
        &self,
        _update: &nodus_client_core::model::ProfileUpdate,
    ) -> impl core::future::Future<Output = Result<Account, Self::Error>> {
        fail::<Account>()
    }
}

impl nodus_client_core::actions::Search for RichInstance {
    type Error = Unavailable;

    fn search(
        &self,
        _query: &str,
        _resolve: bool,
    ) -> impl core::future::Future<Output = Result<nodus_client_core::model::SearchResults, Self::Error>>
    {
        core::future::ready(Ok(self.search_results.clone()))
    }
}

impl nodus_client_core::actions::ScheduleActions for RichInstance {
    type Error = Unavailable;

    fn scheduled(
        &self,
    ) -> impl core::future::Future<
        Output = Result<Vec<nodus_client_core::model::ScheduledStatus>, Self::Error>,
    > {
        fail::<Vec<nodus_client_core::model::ScheduledStatus>>()
    }

    fn schedule(
        &self,
        _draft: &Draft,
        _scheduled_at: &str,
    ) -> impl core::future::Future<Output = Result<nodus_client_core::model::ScheduledStatus, Self::Error>>
    {
        fail::<nodus_client_core::model::ScheduledStatus>()
    }

    fn cancel_scheduled(
        &self,
        _id: &str,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }
}

impl nodus_client_core::actions::FilterActions for RichInstance {
    type Error = Unavailable;

    fn filters(
        &self,
    ) -> impl core::future::Future<Output = Result<Vec<nodus_client_core::model::Filter>, Self::Error>>
    {
        fail::<Vec<nodus_client_core::model::Filter>>()
    }

    fn create_filter(
        &self,
        _draft: &nodus_client_core::model::FilterDraft,
    ) -> impl core::future::Future<Output = Result<nodus_client_core::model::Filter, Self::Error>>
    {
        fail::<nodus_client_core::model::Filter>()
    }

    fn update_filter(
        &self,
        _id: &str,
        _draft: &nodus_client_core::model::FilterDraft,
    ) -> impl core::future::Future<Output = Result<nodus_client_core::model::Filter, Self::Error>>
    {
        fail::<nodus_client_core::model::Filter>()
    }

    fn delete_filter(
        &self,
        _id: &str,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }
}

impl nodus_client_core::actions::HighlightActions for RichInstance {
    type Error = Unavailable;

    fn featured_tags(
        &self,
        _id: &LocalId,
    ) -> impl core::future::Future<
        Output = Result<Vec<nodus_client_core::model::FeaturedTag>, Self::Error>,
    > {
        fail::<Vec<nodus_client_core::model::FeaturedTag>>()
    }

    fn feature_tag(
        &self,
        _name: &str,
    ) -> impl core::future::Future<Output = Result<nodus_client_core::model::FeaturedTag, Self::Error>>
    {
        fail::<nodus_client_core::model::FeaturedTag>()
    }

    fn unfeature_tag(
        &self,
        _id: &str,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }

    fn rules(
        &self,
    ) -> impl core::future::Future<Output = Result<Vec<nodus_client_core::model::Rule>, Self::Error>>
    {
        fail::<Vec<nodus_client_core::model::Rule>>()
    }
}

struct PagedDiscovery {
    page1: AccountFeed,
    page2: AccountFeed,
}

impl Discovery for PagedDiscovery {
    type Error = Unavailable;

    fn meta(&self) -> impl core::future::Future<Output = Result<Meta, Self::Error>> {
        fail::<Meta>()
    }

    fn me(&self) -> impl core::future::Future<Output = Result<MeResponse, Self::Error>> {
        fail::<MeResponse>()
    }

    fn feed(
        &self,
        cursor: Option<&str>,
    ) -> impl core::future::Future<Output = Result<AccountFeed, Self::Error>> {
        let answer = if cursor.is_some() {
            self.page2.clone()
        } else {
            self.page1.clone()
        };
        core::future::ready(Ok(answer))
    }

    fn posts(
        &self,
        _cursor: Option<&str>,
    ) -> impl core::future::Future<Output = Result<PostFeed, Self::Error>> {
        fail::<PostFeed>()
    }

    fn refresh(
        &self,
        _feed_id: &str,
    ) -> impl core::future::Future<Output = Result<AccountFeed, Self::Error>> {
        fail::<AccountFeed>()
    }

    fn send_events(
        &self,
        _batch: &EventBatch,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }

    fn set_sync(
        &self,
        _enable: bool,
        _telemetry_opt_in: bool,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }
}

impl Discovery for FeedProbe {
    type Error = Unavailable;

    fn meta(&self) -> impl core::future::Future<Output = Result<Meta, Self::Error>> {
        fail::<Meta>()
    }

    fn me(&self) -> impl core::future::Future<Output = Result<MeResponse, Self::Error>> {
        fail::<MeResponse>()
    }

    fn feed(
        &self,
        _cursor: Option<&str>,
    ) -> impl core::future::Future<Output = Result<AccountFeed, Self::Error>> {
        self.calls.set(self.calls.get() + 1);
        if self.fail.get() {
            core::future::ready(Err(Unavailable))
        } else {
            let (account_feed, _) = sample_feeds();
            core::future::ready(Ok(account_feed))
        }
    }

    fn posts(
        &self,
        _cursor: Option<&str>,
    ) -> impl core::future::Future<Output = Result<PostFeed, Self::Error>> {
        fail::<PostFeed>()
    }

    fn refresh(
        &self,
        _feed_id: &str,
    ) -> impl core::future::Future<Output = Result<AccountFeed, Self::Error>> {
        fail::<AccountFeed>()
    }

    fn send_events(
        &self,
        _batch: &EventBatch,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }

    fn set_sync(
        &self,
        _enable: bool,
        _telemetry_opt_in: bool,
    ) -> impl core::future::Future<Output = Result<(), Self::Error>> {
        fail::<()>()
    }
}

#[test]
fn timeline_request_list_fetches_list_timeline_and_upserts_reblog_accounts() {
    let instance = RichInstance {
        list_statuses: vec![boosted_status("boost-1", "booster-1", "orig-1", "author-1")],
        ..RichInstance::default()
    };
    let sdk = Sdk::new(
        instance,
        Consented(SyncConsent::default()),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );

    block_on(sdk.load_timeline(TimelineRequest::List("list-42"), None, 20))
        .expect("list timeline succeeds");

    let (list_statuses, _) = sdk.read_timeline(&TimelineRequest::List("list-42"));
    assert_eq!(
        list_statuses
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        vec!["boost-1"]
    );
    assert_eq!(sdk.read_timeline(&TimelineRequest::Home).0, Vec::new());
    assert!(sdk.read_account("booster-1").is_some());
    assert!(
        sdk.read_account("author-1").is_some(),
        "reblogged status author must also be upserted"
    );
}

#[test]
fn load_list_page_upserts_accounts_including_reblog_authors() {
    let instance = RichInstance {
        list_statuses: vec![boosted_status("boost-2", "booster-2", "orig-2", "author-2")],
        ..RichInstance::default()
    };
    let sdk = Sdk::new(
        instance,
        Consented(SyncConsent::default()),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );

    block_on(sdk.load_list_page("list-7", None, 20)).expect("list page succeeds");

    assert!(sdk.read_account("booster-2").is_some());
    assert!(sdk.read_account("author-2").is_some());
    assert!(sdk.read_status("boost-2").is_some());
}

#[test]
fn status_removed_prunes_non_timeline_pages() {
    use nodus_state::page::{PageItems, PageKind};

    let sdk = sdk_with(SyncConsent::default());
    sdk.adopt_items(
        PageKind::Bookmarks,
        PageItems::Statuses(vec![status("keep"), status("gone")]),
        None,
    );
    sdk.adopt_items(
        PageKind::Notifications,
        PageItems::Notifications(vec![Notification {
            id: "n1".to_owned(),
            kind: nodus_client_core::model::NotificationKind::Mention,
            created_at: "2026-11-07T00:00:00.000Z".to_owned(),
            account: account("acct-n1"),
            status: Some(status("gone")),
        }]),
        None,
    );

    sdk.dispatch(Command::StatusRemoved("gone".to_owned()));

    let bookmarks = sdk.page(&PageKind::Bookmarks).expect("bookmarks page");
    if let PageItems::Statuses(items) = bookmarks.items {
        assert_eq!(
            items.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            vec!["keep"]
        );
    } else {
        panic!("expected statuses page");
    }
    let notes = sdk
        .page(&PageKind::Notifications)
        .expect("notifications page");
    if let PageItems::Notifications(items) = notes.items {
        assert_eq!(items, Vec::new());
    } else {
        panic!("expected notifications page");
    }
}

#[test]
fn load_feed_with_cursor_extends_and_keeps_feed_id() {
    let (mut page1, _) = sample_feeds();
    page1.next_cursor = Some("c1".to_owned());
    let mut page2 = page1.clone();
    page2.feed_id = "af-2".to_owned();
    page2.next_cursor = None;
    page2.items[0].item_id = 11;
    page2.items[0].account.id = "carol-1".to_owned();

    let sdk = Sdk::new(
        UnavailableBackend,
        PagedDiscovery { page1, page2 },
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );

    block_on(sdk.load_feed(None)).expect("first page");
    block_on(sdk.load_feed(Some("c1"))).expect("second page");

    let feed = sdk.state().feed.get();
    assert_eq!(feed.feed_id.as_deref(), Some("af-1"));
    assert_eq!(
        feed.items.iter().map(|i| i.item_id).collect::<Vec<_>>(),
        vec![10, 11]
    );
    assert_eq!(feed.next_cursor, None);
}

#[test]
fn apply_handles_reblog_accounts_conversations_and_filters_changed() {
    use nodus_client_core::StreamEvent;
    use nodus_state::page::{PageItems, PageKind};

    let sdk = sdk_with(SyncConsent::default());

    let boosted = boosted_status("s1", "booster", "s0", "reblog-author");
    assert!(sdk.apply(&StreamEvent::Update(Box::new(boosted))));
    assert!(sdk.read_account("booster").is_some());
    assert!(sdk.read_account("reblog-author").is_some());

    let (dm_status, mut conversation) = sample_dm_conversation();
    sdk.adopt_items(
        PageKind::Conversations,
        PageItems::Conversations(vec![conversation.clone()]),
        None,
    );
    conversation.unread = false;
    conversation.accounts.push(account("extra-participant"));
    let updated_dm = boosted_status("dm-2", "alice-dm", "dm-orig", "dm-orig-author");
    conversation.last_status = Some(updated_dm);
    assert!(sdk.apply(&StreamEvent::Conversation(Box::new(conversation))));

    assert!(sdk.read_account("extra-participant").is_some());
    assert!(sdk.read_account("dm-orig-author").is_some());
    assert!(sdk.read_status("dm-2").is_some());
    let conv_page = sdk
        .page(&PageKind::Conversations)
        .expect("conversations page");
    if let PageItems::Conversations(items) = conv_page.items {
        assert!(!items[0].unread);
        assert_eq!(
            items[0].last_status.as_ref().map(|s| s.id.as_str()),
            Some("dm-2")
        );
    } else {
        panic!("expected conversations page");
    }
    let _ = dm_status;

    sdk.adopt_items(PageKind::Filters, PageItems::Filters(Vec::new()), None);
    assert!(
        sdk.page(&PageKind::Filters)
            .and_then(|e| e.fetched_at)
            .is_some()
    );
    assert!(sdk.apply(&StreamEvent::FiltersChanged));
    assert!(
        sdk.page(&PageKind::Filters)
            .and_then(|e| e.fetched_at)
            .is_none()
    );
}

#[test]
fn absorb_upserts_accounts_and_statuses_for_browse_pages() {
    use nodus_state::page::PageKind;

    let instance = RichInstance {
        favourites_statuses: vec![boosted_status(
            "fav-1",
            "fav-booster",
            "fav-orig",
            "fav-author",
        )],
        notifications: vec![Notification {
            id: "note-1".to_owned(),
            kind: nodus_client_core::model::NotificationKind::Reblog,
            created_at: "2026-11-07T00:00:00.000Z".to_owned(),
            account: account("note-actor"),
            status: Some(boosted_status(
                "note-status",
                "note-booster",
                "note-orig",
                "note-orig-author",
            )),
        }],
        conversations: vec![nodus_client_core::model::Conversation {
            id: "conv-absorb".to_owned(),
            unread: true,
            accounts: vec![account("conv-peer")],
            last_status: Some(status("conv-last")),
        }],
        search_results: nodus_client_core::model::SearchResults {
            accounts: vec![account("search-acct")],
            statuses: vec![boosted_status(
                "search-status",
                "search-status-author",
                "search-orig",
                "search-orig-author",
            )],
            hashtags: Vec::new(),
        },
        ..RichInstance::default()
    };

    let sdk = Sdk::new(
        instance,
        Consented(SyncConsent::default()),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );

    block_on(sdk.ensure_page(PageKind::Favourites, None, 20, 60_000)).expect("favourites");
    assert!(sdk.read_account("fav-booster").is_some());
    assert!(sdk.read_account("fav-author").is_some());
    assert!(sdk.read_status("fav-1").is_some());

    block_on(sdk.ensure_page(PageKind::Notifications, None, 20, 60_000)).expect("notifications");
    assert!(sdk.read_account("note-actor").is_some());
    assert!(sdk.read_account("note-orig-author").is_some());
    assert!(sdk.read_status("note-status").is_some());

    block_on(sdk.ensure_page(PageKind::Conversations, None, 20, 60_000)).expect("conversations");
    assert!(sdk.read_account("conv-peer").is_some());
    assert!(sdk.read_status("conv-last").is_some());

    block_on(sdk.ensure_page(PageKind::Search("rust".to_owned()), None, 20, 60_000))
        .expect("search");
    assert!(sdk.read_account("search-acct").is_some());
    assert!(sdk.read_account("search-status-author").is_some());
    assert!(sdk.read_account("search-orig-author").is_some());
    assert!(sdk.read_status("search-status").is_some());
}

fn signed_in_session() -> nodus_state::Session {
    nodus_state::Session {
        host: "remote.test".to_owned(),
        account_id: "self-1".to_owned(),
        acct: "someone@remote.test".to_owned(),
        scopes: vec!["read".to_owned()],
    }
}

#[derive(Clone)]
struct SharedCache(std::rc::Rc<MemoryPersistence>);

impl nodus_persist::Persistence for SharedCache {
    fn load(
        &self,
        key: &nodus_persist::CacheKey,
    ) -> impl core::future::Future<Output = Result<Option<Vec<u8>>, nodus_persist::PersistenceError>>
    {
        self.0.load(key)
    }

    fn store(
        &self,
        key: &nodus_persist::CacheKey,
        value: Vec<u8>,
    ) -> impl core::future::Future<Output = Result<(), nodus_persist::PersistenceError>> {
        self.0.store(key, value)
    }

    fn remove(
        &self,
        key: &nodus_persist::CacheKey,
    ) -> impl core::future::Future<Output = Result<(), nodus_persist::PersistenceError>> {
        self.0.remove(key)
    }

    fn clear(
        &self,
    ) -> impl core::future::Future<Output = Result<(), nodus_persist::PersistenceError>> {
        self.0.clear()
    }
}

#[test]
fn restore_home_timeline_restores_statuses_so_reload_within_ttl_is_not_blank() {
    let cache = SharedCache(std::rc::Rc::new(MemoryPersistence::new()));

    // First session loads Home and caches it.
    let first_sdk = Sdk::new(
        Pages::of(vec![vec!["1", "2"]]),
        Consented(SyncConsent::default()),
        cache.clone(),
        FixedClock::at(1_000),
    );
    block_on(first_sdk.load_timeline(TimelineRequest::Home, None, 20)).expect("initial load");
    block_on(first_sdk.cache_home_timeline("remote.test")).expect("cache home timeline");

    // Second session (reload within TTL) restores from persistence and runs ensure_timeline.
    let second_instance = Pages::of(vec![vec!["9"]]);
    let second_sdk = Sdk::new(
        second_instance,
        Consented(SyncConsent::default()),
        cache,
        FixedClock::at(5_000),
    );
    block_on(second_sdk.restore_home_timeline("remote.test"));

    let (restored, _) = second_sdk.read_timeline(&TimelineRequest::Home);
    assert_eq!(
        restored
            .iter()
            .map(|status| status.id.as_str())
            .collect::<Vec<_>>(),
        vec!["1", "2"],
        "warm-started home timeline must render cached statuses instead of a blank list",
    );

    let ensured = block_on(second_sdk.ensure_timeline(TimelineRequest::Home, None, 20, 60_000))
        .expect("ensure succeeds");
    assert!(
        ensured.is_none(),
        "restored timeline with statuses within TTL should not refetch",
    );
}

#[test]
fn ensure_timeline_refetches_when_page_is_fresh_but_statuses_are_missing() {
    use nodus_persist::{CacheKey, Persistence};
    use nodus_state::{Timeline, TimelinePage};

    let cache = MemoryPersistence::new();
    let stale_only_ids = TimelinePage {
        status_ids: vec!["1".to_owned(), "2".to_owned()],
        max_id: Some("2".to_owned()),
        fetched_at: Some(1_000),
    };
    block_on(cache.store(
        &CacheKey::Timeline {
            host: "remote.test".to_owned(),
            which: Timeline::Home,
            tag: None,
        },
        serde_json::to_vec(&stale_only_ids).expect("serialize page"),
    ))
    .expect("store timeline page");

    let sdk = Sdk::new(
        Pages::of(vec![vec!["1", "2"]]),
        Consented(SyncConsent::default()),
        cache,
        FixedClock::at(5_000),
    );
    block_on(sdk.restore_home_timeline("remote.test"));

    let ensured = block_on(sdk.ensure_timeline(TimelineRequest::Home, None, 20, 60_000))
        .expect("ensure refetches when entity store lacks statuses");
    assert!(
        ensured.is_some(),
        "must fetch from network when timeline ids are missing from statuses store",
    );
    assert_eq!(sdk.read_timeline(&TimelineRequest::Home).0.len(), 2);
}

#[test]
fn drive_does_not_refresh_feed_when_signed_out() {
    let sdk = Sdk::new(
        Pages::of(vec![vec!["1"]]),
        FeedProbe::new(false),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );
    let mut driver = crate::driver::Driver::new();
    let now = web_time::Instant::now();

    let report = block_on(sdk.drive(&mut driver, now));

    assert_eq!(report, crate::sdk::Drive::default());
    assert!(driver.budget().last_attempt.is_none());
}

#[test]
fn drive_refreshes_feed_at_ten_minutes_despite_five_minute_timeline_refreshes() {
    use crate::scheduler::{FEED_STALE, TIMELINE_STALE};
    use std::time::Duration;

    let sdk = Sdk::new(
        Pages::of(vec![vec!["1"], vec!["2"], vec!["3"]]),
        FeedProbe::new(false),
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );
    sdk.adopt_session(signed_in_session());
    let mut driver = crate::driver::Driver::new();
    let t0 = web_time::Instant::now();

    // Boot: both timeline and feed load.
    let first = block_on(sdk.drive(&mut driver, t0));
    assert!(first.timeline);
    assert!(first.feed);

    // At +5m: timeline refreshes, feed is still fresh.
    let t5 = t0 + TIMELINE_STALE;
    let second = block_on(sdk.drive(&mut driver, t5));
    assert!(second.timeline);
    assert!(!second.feed);

    // At +10m: both timeline (5m since +5m) and feed (10m since t0) must refresh.
    let t10 = t0 + FEED_STALE;
    let third = block_on(sdk.drive(&mut driver, t10));
    assert!(third.timeline);
    assert!(
        third.feed,
        "5m timeline refresh must not starve the 10m feed refresh",
    );

    // Also verify 5m+1s is not enough for feed if timeline refreshed at 5m.
    let t11 = t10 + Duration::from_secs(1);
    let fourth = block_on(sdk.drive(&mut driver, t11));
    assert!(!fourth.timeline);
    assert!(!fourth.feed);
}

#[test]
fn drive_and_next_wake_respect_exponential_retry_backoff_on_outage() {
    use crate::scheduler::RETRY_BASE;
    use std::time::Duration;

    let sdk = Sdk::new(
        UnavailableBackend,
        UnavailableBackend,
        MemoryPersistence::new(),
        FixedClock::at(1_000),
    );
    sdk.adopt_session(signed_in_session());
    let mut driver = crate::driver::Driver::new();
    driver.record_stream(crate::scheduler::StreamPhase::Connected);
    let t0 = web_time::Instant::now();

    // First failure at t0 -> failures = 1, retry_in = 4s.
    let first = block_on(sdk.drive(&mut driver, t0));
    assert!(first.failed);
    assert_eq!(driver.budget().failures, 1);

    // At +1s (MIN_ATTEMPT_GAP), backoff (4s) has NOT elapsed.
    let t1 = t0 + Duration::from_secs(1);
    assert_eq!(
        driver.next_wake(t1),
        Some(Duration::from_secs(3)),
        "must wait remaining retry backoff instead of returning Duration::ZERO at 1s",
    );
    let at_1s = block_on(sdk.drive(&mut driver, t1));
    assert!(
        at_1s.is_empty(),
        "must not retry after 1s while 4s backoff is active",
    );

    // At +4s (RETRY_BASE * 2^1), retry is due again -> fails again (failures = 2, retry_in = 8s).
    let t4 = t0 + RETRY_BASE * 2;
    assert_eq!(driver.next_wake(t4), Some(Duration::ZERO));
    let at_4s = block_on(sdk.drive(&mut driver, t4));
    assert!(at_4s.failed);
    assert_eq!(driver.budget().failures, 2);

    // At t4 + 4s, still waiting for 8s backoff.
    let t4_plus_4 = t4 + Duration::from_secs(4);
    assert_eq!(driver.next_wake(t4_plus_4), Some(Duration::from_secs(4)));
    assert!(block_on(sdk.drive(&mut driver, t4_plus_4)).is_empty());
}
