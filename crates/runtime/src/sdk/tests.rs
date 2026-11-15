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
