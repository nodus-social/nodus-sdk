//! `InstanceApi` for every transport, written once.

mod actions;
mod admin;
mod filters;
mod filters_v1;
mod highlights;
mod markers;
mod meta;
mod registration;
mod schedule;
mod settings;
mod tags;
#[cfg(test)]
mod tests;

use crate::actions::{
    AccountActions, Browse, FilterActions, HighlightActions, MarkerActions, Search, SettingsActions,
};
use crate::api::InstanceApi;
use crate::model::{
    Account, Announcement, Context, Conversation, Draft, FeaturedTag, Filter, List, LocalId,
    MarkerTimeline, Markers, Notification, Page, PageParams, Preferences, Relationship,
    SearchResults, Status, StreamKind,
};
use crate::rest::{RestInstance, Transport};
use crate::{Error, Result};

impl<T: Transport> InstanceApi for RestInstance<T> {
    type Error = Error;

    async fn credential_account(&self) -> Result<crate::model::CredentialAccount> {
        let url = self.url("/api/v1/accounts/verify_credentials")?;
        self.fetch(self.get(url)).await
    }

    async fn verify_credentials(&self) -> Result<Account> {
        let url = self.url("/api/v1/accounts/verify_credentials")?;
        self.fetch(self.get(url)).await
    }

    async fn timeline(&self, kind: StreamKind, params: PageParams) -> Result<Page<Status>> {
        let (path, local) = match kind {
            StreamKind::User => ("/api/v1/timelines/home", false),
            StreamKind::Local => ("/api/v1/timelines/public", true),
            StreamKind::Federated => ("/api/v1/timelines/public", false),
        };
        let mut url = self.page_url(path, &params)?;
        if local {
            url.query_pairs_mut().append_pair("local", "true");
        }
        self.fetch_page(self.get(url), &params).await
    }

    async fn tag_timeline(&self, tag: &str, params: PageParams) -> Result<Page<Status>> {
        let url = self.page_url_segments(&["api", "v1", "timelines", "tag", tag], &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn list_timeline(&self, list_id: &str, params: PageParams) -> Result<Page<Status>> {
        let url = self.page_url_segments(&["api", "v1", "timelines", "list", list_id], &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn status(&self, id: &str) -> Result<Status> {
        let url = self.url_segments(&["api", "v1", "statuses", id])?;
        self.fetch(self.get(url)).await
    }

    async fn status_context(&self, id: &str) -> Result<Context> {
        let url = self.url_segments(&["api", "v1", "statuses", id, "context"])?;
        self.fetch(self.get(url)).await
    }

    async fn notifications(&self, params: PageParams) -> Result<Page<Notification>> {
        let url = self.page_url("/api/v1/notifications", &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn account(&self, id: &LocalId) -> Result<Account> {
        let url = self.url_segments(&["api", "v1", "accounts", &id.0])?;
        self.fetch(self.get(url)).await
    }

    async fn account_lookup(&self, acct: &str) -> Result<Account> {
        let mut url = self.url("/api/v1/accounts/lookup")?;
        url.query_pairs_mut().append_pair("acct", acct);
        self.fetch(self.get(url)).await
    }

    async fn account_followers(&self, id: &LocalId, params: PageParams) -> Result<Page<Account>> {
        let url =
            self.page_url_segments(&["api", "v1", "accounts", &id.0, "followers"], &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn account_following(&self, id: &LocalId, params: PageParams) -> Result<Page<Account>> {
        let url =
            self.page_url_segments(&["api", "v1", "accounts", &id.0, "following"], &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn account_statuses(&self, id: &LocalId, params: PageParams) -> Result<Page<Status>> {
        let url = self.page_url_segments(&["api", "v1", "accounts", &id.0, "statuses"], &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn featured_tags(&self, id: &LocalId) -> Result<Vec<FeaturedTag>> {
        HighlightActions::featured_tags(self, id).await
    }

    async fn bookmarks(&self, params: PageParams) -> Result<Page<Status>> {
        Browse::bookmarks(self, params).await
    }

    async fn favourites(&self, params: PageParams) -> Result<Page<Status>> {
        Browse::favourites(self, params).await
    }

    async fn conversations(&self, params: PageParams) -> Result<Page<Conversation>> {
        Browse::conversations(self, params).await
    }

    async fn lists(&self) -> Result<Vec<List>> {
        Browse::lists(self).await
    }

    async fn follow_requests(&self) -> Result<Vec<Account>> {
        AccountActions::follow_requests(self).await
    }

    async fn mutes(&self, params: PageParams) -> Result<Page<Account>> {
        AccountActions::mutes(self, params).await
    }

    async fn blocks(&self, params: PageParams) -> Result<Page<Account>> {
        AccountActions::blocks(self, params).await
    }

    async fn preferences(&self) -> Result<Preferences> {
        SettingsActions::preferences(self).await
    }

    async fn filters(&self) -> Result<Vec<Filter>> {
        FilterActions::filters(self).await
    }

    async fn markers(&self, timelines: &[MarkerTimeline]) -> Result<Markers> {
        MarkerActions::markers(self, timelines).await
    }

    async fn announcements(&self) -> Result<Vec<Announcement>> {
        SettingsActions::announcements(self).await
    }

    async fn directory(&self, local_only: bool, offset: u32) -> Result<Vec<Account>> {
        Browse::directory(self, local_only, offset).await
    }

    async fn search(&self, query: &str, resolve: bool) -> Result<SearchResults> {
        Search::search(self, query, resolve).await
    }

    async fn relationship(&self, id: &LocalId) -> Result<Relationship> {
        let mut url = self.url("/api/v1/accounts/relationships")?;
        url.query_pairs_mut().append_pair("id[]", &id.0);
        let mut found: Vec<Relationship> = self.fetch(self.get(url)).await?;
        found
            .pop()
            .ok_or_else(|| Error::Decode("empty relationship list".to_owned()))
    }

    async fn follow(&self, id: &LocalId) -> Result<Relationship> {
        let url = self.url_segments(&["api", "v1", "accounts", &id.0, "follow"])?;
        self.fetch(self.post(url)).await
    }

    async fn unfollow(&self, id: &LocalId) -> Result<Relationship> {
        let url = self.url_segments(&["api", "v1", "accounts", &id.0, "unfollow"])?;
        self.fetch(self.post(url)).await
    }

    async fn post_status(&self, draft: &Draft) -> Result<Status> {
        let url = self.url("/api/v1/statuses")?;
        let form = status_form(draft, false);
        let request = attach_idempotency_key(self.post_form(url, &form), draft);
        self.fetch(request).await
    }

    async fn delete_status(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "statuses", id])?;
        self.send_empty(self.delete(url)).await
    }
}

/// Builds the form fields for posting, scheduling, or editing a status.
///
/// When `for_edit` is `true`, `spoiler_text` and `sensitive` are always emitted even
/// when empty or `false` so an edit can clear a content warning or unmark sensitivity,
/// and `in_reply_to_id` is omitted because `PUT /api/v1/statuses/:id` does not accept it.
pub(super) fn status_form(draft: &Draft, for_edit: bool) -> Vec<(&'static str, String)> {
    let mut form: Vec<(&'static str, String)> = vec![
        ("status", draft.text.clone()),
        ("visibility", draft.visibility.as_str().to_owned()),
    ];
    if !for_edit {
        if let Some(reply_to) = &draft.in_reply_to_id {
            form.push(("in_reply_to_id", reply_to.clone()));
        }
    }
    for media_id in &draft.media_ids {
        form.push(("media_ids[]", media_id.clone()));
    }
    if for_edit || !draft.spoiler_text.is_empty() {
        form.push(("spoiler_text", draft.spoiler_text.clone()));
    }
    if let Some(language) = draft.language.as_ref() {
        form.push(("language", language.clone()));
    }
    if for_edit {
        form.push(("sensitive", draft.sensitive.to_string()));
    } else if draft.sensitive {
        form.push(("sensitive", "true".to_owned()));
    }
    if let Some(poll) = &draft.poll {
        // The wire shape is `poll[options][]`, repeated; an option that is only spaces
        // is dropped here rather than sent, because the instance rejects the whole post.
        for option in poll
            .options
            .iter()
            .filter(|option| !option.trim().is_empty())
        {
            form.push(("poll[options][]", option.clone()));
        }
        form.push(("poll[expires_in]", poll.expires_in.to_string()));
        if poll.multiple {
            form.push(("poll[multiple]", "true".to_owned()));
        }
        if poll.hide_totals {
            form.push(("poll[hide_totals]", "true".to_owned()));
        }
    }
    form
}

/// Attaches the draft's `Idempotency-Key` header if present.
pub(super) fn attach_idempotency_key(
    mut request: crate::rest::HttpRequest,
    draft: &Draft,
) -> crate::rest::HttpRequest {
    // Idempotency is what stops a retried compose from posting twice; a draft
    // without a key is a caller bug, not something to paper over here.
    if let Some(key) = &draft.idempotency_key {
        request
            .headers
            .push(("Idempotency-Key".to_owned(), key.clone()));
    }
    request
}

#[cfg(feature = "native")]
mod sync_impl {
    use crate::Result;
    use crate::model::{Account, LocalId, Page, PageParams, Status};
    use crate::rest::{RestInstance, Transport};
    use crate::sync_api::SyncApi;

    impl<T: Transport> SyncApi for RestInstance<T> {
        type Error = crate::Error;

        async fn following(&self, id: &LocalId, params: PageParams) -> Result<Page<Account>> {
            let url =
                self.page_url_segments(&["api", "v1", "accounts", &id.0, "following"], &params)?;
            self.fetch_page(self.get(url), &params).await
        }

        async fn favourites(&self, params: PageParams) -> Result<Page<Status>> {
            let url = self.page_url("/api/v1/favourites", &params)?;
            self.fetch_page(self.get(url), &params).await
        }

        async fn bookmarks(&self, params: PageParams) -> Result<Page<Status>> {
            let url = self.page_url("/api/v1/bookmarks", &params)?;
            self.fetch_page(self.get(url), &params).await
        }
    }
}

#[cfg(test)]
mod url_tests {
    use core::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::rest::{HttpRequest, HttpResponse};

    #[derive(Clone, Default)]
    struct Recording(Rc<RefCell<Vec<url::Url>>>);

    impl Transport for Recording {
        fn send(
            &self,
            request: HttpRequest,
        ) -> impl core::future::Future<Output = Result<HttpResponse>> {
            self.0.borrow_mut().push(request.url);
            core::future::ready(Ok(HttpResponse {
                status: 200,
                retry_after_s: None,
                body: b"[]".to_vec(),
            }))
        }
    }

    fn instance_at(base: &str) -> (RestInstance<Recording>, Rc<RefCell<Vec<url::Url>>>) {
        let recording = Recording::default();
        let seen = Rc::clone(&recording.0);
        let instance = crate::Instance::parse(base).expect("valid host");
        (
            RestInstance::with_transport(&instance, "token", recording),
            seen,
        )
    }

    fn block_on<F: core::future::Future>(future: F) -> F::Output {
        let mut future = core::pin::pin!(future);
        let mut context = core::task::Context::from_waker(core::task::Waker::noop());
        match future.as_mut().poll(&mut context) {
            core::task::Poll::Ready(output) => output,
            core::task::Poll::Pending => panic!("recording transport never pends"),
        }
    }

    #[test]
    fn tag_timeline_encodes_hash_instead_of_treating_it_as_fragment() {
        let (client, seen) = instance_at("https://mastodon.example/prefix");
        let _ = block_on(client.tag_timeline("#rust", PageParams::default()));
        let url = &seen.borrow()[0];
        assert_eq!(url.fragment(), None);
        assert_eq!(url.path(), "/prefix/api/v1/timelines/tag/%23rust");
    }

    #[test]
    fn status_and_account_ids_cannot_traverse_or_inject_query_or_fragment() {
        let (client, seen) = instance_at("https://mastodon.example");
        let _ = block_on(client.status("../admin?foo=bar#frag"));
        let _ = block_on(client.status_context("../admin?foo=bar#frag"));
        let _ = block_on(
            client.account_followers(&LocalId("../x#y".to_owned()), PageParams::default()),
        );
        let _ = block_on(client.delete_status("123?admin=1#x"));

        let urls = seen.borrow();
        assert_eq!(
            urls[0].path(),
            "/api/v1/statuses/..%2Fadmin%3Ffoo=bar%23frag"
        );
        assert_eq!(urls[0].query(), None);
        assert_eq!(urls[0].fragment(), None);

        assert_eq!(
            urls[1].path(),
            "/api/v1/statuses/..%2Fadmin%3Ffoo=bar%23frag/context"
        );
        assert_eq!(urls[2].path(), "/api/v1/accounts/..%2Fx%23y/followers");
        assert_eq!(urls[3].path(), "/api/v1/statuses/123%3Fadmin=1%23x");
    }

    #[test]
    fn dot_and_empty_segments_are_rejected() {
        let (client, _) = instance_at("https://mastodon.example");
        for bad in ["", ".", ".."] {
            assert!(block_on(client.status(bad)).is_err());
            assert!(block_on(client.tag_timeline(bad, PageParams::default())).is_err());
            assert!(block_on(client.account(&LocalId(bad.to_owned()))).is_err());
        }
    }
}
