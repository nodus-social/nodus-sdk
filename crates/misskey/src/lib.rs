//! Misskey's API, mapped onto the same [`InstanceApi`] the Mastodon crate implements.
//!
//! Misskey is not a Mastodon dialect: every call is a JSON POST, ids are opaque
//! strings, visibility has five values, and notes carry MFM rather than HTML. The
//! mapping lives here so the SDK state machine and the views never branch on the
//! server software.

#![allow(
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    clippy::format_push_string,
    clippy::cast_possible_truncation,
    clippy::unused_self,
    reason = "Misskey wire values are consumed by the mapper, and the port signatures are the Mastodon ones"
)]

pub mod convert;

use core::fmt;

use nodus_client_core::model::{
    Account, Context, CredentialAccount, Draft, LocalId, Notification, Page, PageParams,
    Relationship, Status, StreamKind,
};
use nodus_client_core::rest::{HttpRequest, HttpResponse, Method, Transport};
use nodus_client_core::{Error, Instance, Result};
use serde_json::{Value, json};
use url::Url;

pub use convert::note_html;

/// One Misskey instance, one token, one transport.
pub struct MisskeyInstance<T: Transport> {
    base: Url,
    token: String,
    transport: T,
}

impl<T: Transport> fmt::Debug for MisskeyInstance<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MisskeyInstance")
            .field("base", &self.base.as_str())
            .field("token", &"<redacted>")
            .finish_non_exhaustive()
    }
}

impl<T: Transport> Clone for MisskeyInstance<T> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            token: self.token.clone(),
            transport: self.transport.clone(),
        }
    }
}

impl<T: Transport> MisskeyInstance<T> {
    /// Build from an already-parsed instance and a transport.
    #[must_use]
    pub fn with_transport(instance: &Instance, token: impl Into<String>, transport: T) -> Self {
        Self {
            base: instance.base.clone(),
            token: token.into(),
            transport,
        }
    }

    /// The instance this client talks to.
    #[must_use]
    pub const fn base(&self) -> &Url {
        &self.base
    }

    /// POST one JSON call. Misskey has no GET endpoints.
    async fn call(&self, path: &str, body: Value) -> Result<Value> {
        let url = self.url(path)?;
        let mut headers = vec![("Content-Type".to_owned(), "application/json".to_owned())];
        if !self.token.is_empty() {
            headers.push(("Authorization".to_owned(), format!("Bearer {}", self.token)));
        }
        let request = HttpRequest {
            method: Method::Post,
            url,
            headers,
            body: Some(body.to_string().into_bytes()),
        };
        let response = self.transport.send(request).await?;
        self.finish(response)
    }

    fn finish(&self, response: HttpResponse) -> Result<Value> {
        match response.status {
            200..=299 => serde_json::from_slice(&response.body)
                .map_err(|error| Error::Decode(error.to_string())),
            401 | 403 => Err(Error::Unauthorized),
            429 => Err(Error::RateLimited {
                retry_after_s: response.retry_after_s.unwrap_or(300),
            }),
            status => {
                let body = String::from_utf8_lossy(&response.body);
                let message = serde_json::from_slice::<Value>(&response.body)
                    .ok()
                    .and_then(|value| {
                        value
                            .pointer("/error/message")
                            .or_else(|| value.get("message"))
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    })
                    .unwrap_or_else(|| body.chars().take(200).collect());
                Err(Error::Status { status, message })
            }
        }
    }

    fn url(&self, path: &str) -> Result<Url> {
        let joined = format!("{}{path}", self.base.as_str().trim_end_matches('/'));
        Url::parse(&joined).map_err(|error| Error::Transport(format!("{joined}: {error}")))
    }

    fn body(params: &PageParams) -> Value {
        let mut body = serde_json::Map::new();
        if let Some(limit) = params.limit {
            body.insert("limit".to_owned(), json!(limit));
        }
        if let Some(until) = &params.max_id {
            body.insert("untilId".to_owned(), json!(until));
        }
        if let Some(since) = &params.since_id {
            body.insert("sinceId".to_owned(), json!(since));
        }
        Value::Object(body)
    }

    async fn page<U>(
        &self,
        path: &str,
        params: &PageParams,
        map: impl Fn(Value) -> Option<U>,
    ) -> Result<Page<U>> {
        self.page_with_body(path, Self::body(params), params, map)
            .await
    }

    async fn page_with_body<U>(
        &self,
        path: &str,
        body: Value,
        params: &PageParams,
        map: impl Fn(Value) -> Option<U>,
    ) -> Result<Page<U>> {
        let value = self.call(path, body).await?;
        let array = value.as_array().cloned().unwrap_or_default();
        let next = items_next(&array, params);
        let items: Vec<U> = array.into_iter().filter_map(map).collect();
        Ok(Page::new(items).with_next(next))
    }

    async fn relation(&self, id: &LocalId) -> Result<Relationship> {
        let value = self
            .call("/api/users/relation", json!({ "userId": id.0 }))
            .await?;
        convert::relationship(&value, id)
            .ok_or_else(|| Error::Decode("a relation without the other account".to_owned()))
    }
}

fn items_next(items: &[Value], params: &PageParams) -> Option<String> {
    let full = params
        .limit
        .is_none_or(|limit| items.len() >= usize::from(limit));
    if !full {
        return None;
    }
    items
        .last()
        .and_then(|item| item.get("id"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

impl<T: Transport> nodus_client_core::InstanceApi for MisskeyInstance<T> {
    type Error = Error;

    async fn verify_credentials(&self) -> Result<Account> {
        let value = self.call("/api/i", json!({})).await?;
        convert::account(&value).ok_or_else(|| Error::Decode("i is not an account".to_owned()))
    }

    async fn credential_account(&self) -> Result<CredentialAccount> {
        let value = self.call("/api/i", json!({})).await?;
        let account = convert::account(&value)
            .ok_or_else(|| Error::Decode("i is not an account".to_owned()))?;
        Ok(CredentialAccount {
            account,
            source: convert::source(&value),
        })
    }

    async fn timeline(&self, kind: StreamKind, params: PageParams) -> Result<Page<Status>> {
        let path = match kind {
            StreamKind::User => "/api/notes/timeline",
            StreamKind::Local => "/api/notes/local-timeline",
            StreamKind::Federated => "/api/notes/global-timeline",
        };
        self.page(path, &params, convert::status).await
    }

    async fn tag_timeline(&self, tag: &str, params: PageParams) -> Result<Page<Status>> {
        let mut body = Self::body(&params);
        body["tag"] = json!(tag);
        self.page_with_body("/api/notes/search-by-tag", body, &params, convert::status)
            .await
    }

    async fn list_timeline(&self, list_id: &str, params: PageParams) -> Result<Page<Status>> {
        let mut body = Self::body(&params);
        body["listId"] = json!(list_id);
        self.page_with_body(
            "/api/notes/user-list-timeline",
            body,
            &params,
            convert::status,
        )
        .await
    }

    async fn status(&self, id: &str) -> Result<Status> {
        let value = self
            .call("/api/notes/show", json!({ "noteId": id }))
            .await?;
        convert::status(value).ok_or_else(|| Error::Decode("note is not a status".to_owned()))
    }

    async fn status_context(&self, id: &str) -> Result<Context> {
        let conversation = self
            .call(
                "/api/notes/conversation",
                json!({ "noteId": id, "limit": 40 }),
            )
            .await
            .unwrap_or(Value::Array(Vec::new()));
        let children = self
            .call("/api/notes/children", json!({ "noteId": id, "limit": 40 }))
            .await
            .unwrap_or(Value::Array(Vec::new()));
        Ok(Context {
            ancestors: conversation
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(convert::status)
                .collect(),
            descendants: children
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(convert::status)
                .collect(),
        })
    }

    async fn notifications(&self, params: PageParams) -> Result<Page<Notification>> {
        self.page("/api/i/notifications", &params, convert::notification)
            .await
    }

    async fn account(&self, id: &LocalId) -> Result<Account> {
        let value = self
            .call("/api/users/show", json!({ "userId": id.0 }))
            .await?;
        convert::account(&value).ok_or_else(|| Error::Decode("user is not an account".to_owned()))
    }

    async fn account_lookup(&self, acct: &str) -> Result<Account> {
        let (username, host) = acct
            .split_once('@')
            .map_or((acct, None), |(user, host)| (user, Some(host)));
        let mut body = json!({ "username": username });
        if let Some(host) = host {
            body["host"] = json!(host);
        }
        let value = self.call("/api/users/show", body).await?;
        convert::account(&value).ok_or_else(|| Error::Decode("user is not an account".to_owned()))
    }

    async fn account_followers(&self, id: &LocalId, params: PageParams) -> Result<Page<Account>> {
        let mut body = Self::body(&params);
        body["userId"] = json!(id.0);
        self.page_with_body("/api/users/followers", body, &params, |value| {
            convert::account(value.get("follower").unwrap_or(&value))
        })
        .await
    }

    async fn account_following(&self, id: &LocalId, params: PageParams) -> Result<Page<Account>> {
        let mut body = Self::body(&params);
        body["userId"] = json!(id.0);
        self.page_with_body("/api/users/following", body, &params, |value| {
            convert::account(value.get("followee").unwrap_or(&value))
        })
        .await
    }

    async fn account_statuses(&self, id: &LocalId, params: PageParams) -> Result<Page<Status>> {
        let mut body = Self::body(&params);
        body["userId"] = json!(id.0);
        self.page_with_body("/api/users/notes", body, &params, convert::status)
            .await
    }

    async fn relationship(&self, id: &LocalId) -> Result<Relationship> {
        self.relation(id).await
    }

    async fn follow(&self, id: &LocalId) -> Result<Relationship> {
        self.call("/api/following/create", json!({ "userId": id.0 }))
            .await?;
        self.relation(id).await
    }

    async fn unfollow(&self, id: &LocalId) -> Result<Relationship> {
        self.call("/api/following/delete", json!({ "userId": id.0 }))
            .await?;
        self.relation(id).await
    }

    async fn post_status(&self, draft: &Draft) -> Result<Status> {
        let mut body = json!({
            "text": draft.text,
            "cw": if draft.spoiler_text.is_empty() { Value::Null } else { json!(draft.spoiler_text) },
            "visibility": convert::visibility_out(draft.visibility),
        });
        if let Some(reply) = &draft.in_reply_to_id {
            body["replyId"] = json!(reply);
        }
        if !draft.media_ids.is_empty() {
            body["fileIds"] = json!(draft.media_ids);
        }
        if let Some(poll) = &draft.poll {
            body["poll"] = json!({
                "choices": poll.options,
                "multiple": poll.multiple,
                "expiredAfter": u64::from(poll.expires_in) * 1000,
            });
        }
        let value = self.call("/api/notes/create", body).await?;
        let note = value.get("createdNote").cloned().unwrap_or(value);
        convert::status(note)
            .ok_or_else(|| Error::Decode("created note is not a status".to_owned()))
    }

    async fn delete_status(&self, id: &str) -> Result<()> {
        self.call("/api/notes/delete", json!({ "noteId": id }))
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    use nodus_client_core::InstanceApi;
    use nodus_client_core::model::{Draft, LocalId, PageParams, PollDraft, StreamKind, Visibility};

    use super::*;

    #[derive(Clone)]
    struct RecordedRequest {
        url: String,
        body: Value,
    }

    #[derive(Clone, Default)]
    struct MockTransport {
        requests: Rc<RefCell<Vec<RecordedRequest>>>,
        responses: Rc<RefCell<VecDeque<HttpResponse>>>,
    }

    impl MockTransport {
        fn with_responses(responses: Vec<HttpResponse>) -> Self {
            Self {
                requests: Rc::default(),
                responses: Rc::new(RefCell::new(responses.into())),
            }
        }
    }

    impl Transport for MockTransport {
        fn send(
            &self,
            request: HttpRequest,
        ) -> impl core::future::Future<Output = Result<HttpResponse>> {
            let body = request
                .body
                .as_deref()
                .and_then(|bytes| serde_json::from_slice(bytes).ok())
                .unwrap_or(Value::Null);
            self.requests.borrow_mut().push(RecordedRequest {
                url: request.url.to_string(),
                body,
            });
            let response = self
                .responses
                .borrow_mut()
                .pop_front()
                .unwrap_or(HttpResponse {
                    status: 200,
                    retry_after_s: None,
                    body: b"[]".to_vec(),
                });
            core::future::ready(Ok(response))
        }
    }

    fn instance_with(
        responses: Vec<HttpResponse>,
    ) -> (
        MisskeyInstance<MockTransport>,
        Rc<RefCell<Vec<RecordedRequest>>>,
    ) {
        let transport = MockTransport::with_responses(responses);
        let requests = Rc::clone(&transport.requests);
        let instance = Instance::parse("https://misskey.example").expect("valid host");
        (
            MisskeyInstance::with_transport(&instance, "token", transport),
            requests,
        )
    }

    fn ok_json(json: &Value) -> HttpResponse {
        HttpResponse {
            status: 200,
            retry_after_s: None,
            body: json.to_string().into_bytes(),
        }
    }

    fn block_on<F: core::future::Future>(future: F) -> F::Output {
        let mut future = core::pin::pin!(future);
        let mut context = core::task::Context::from_waker(core::task::Waker::noop());
        match future.as_mut().poll(&mut context) {
            core::task::Poll::Ready(output) => output,
            core::task::Poll::Pending => panic!("the mock transport never pends"),
        }
    }

    fn sample_note(id: &str) -> Value {
        json!({
            "id": id,
            "createdAt": "2026-11-12T03:00:00.000Z",
            "text": "hello",
            "user": { "id": "u1", "username": "alice" }
        })
    }

    #[test]
    fn post_status_with_poll_uses_relative_expired_after() {
        let response = json!({
            "createdNote": sample_note("note-1")
        });
        let (client, requests) = instance_with(vec![ok_json(&response)]);
        let mut draft = Draft::text("pick one", Visibility::Public);
        draft.poll =
            Some(PollDraft::new(vec!["A".to_owned(), "B".to_owned()], 3600).multiple(true));

        let _status = block_on(client.post_status(&draft)).expect("post_status");
        let sent = &requests.borrow()[0];
        assert_eq!(sent.url, "https://misskey.example/api/notes/create");
        let poll = &sent.body["poll"];
        assert_eq!(poll["choices"], json!(["A", "B"]));
        assert_eq!(poll["multiple"], json!(true));
        assert_eq!(poll["expiredAfter"], json!(3_600_000));
        assert!(poll.get("expiresAt").is_none());
    }

    #[test]
    fn tag_timeline_calls_search_by_tag_and_omits_cursor_on_short_page() {
        let notes = json!([sample_note("n1"), sample_note("n2")]);
        let (client, requests) = instance_with(vec![ok_json(&notes)]);

        let page =
            block_on(client.tag_timeline("rust", PageParams::first(5))).expect("tag_timeline");
        let sent = &requests.borrow()[0];
        assert_eq!(sent.url, "https://misskey.example/api/notes/search-by-tag");
        assert_eq!(sent.body["tag"], json!("rust"));
        assert_eq!(sent.body["limit"], json!(5));
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.next_max_id, None);
    }

    #[test]
    fn pagination_returns_cursor_only_when_page_reaches_limit() {
        let two_notes = json!([sample_note("n1"), sample_note("n2")]);
        let two_followers = json!([
            { "id": "rel1", "follower": { "id": "u1", "username": "alice" } },
            { "id": "rel2", "follower": { "id": "u2", "username": "bob" } }
        ]);
        let two_following = json!([
            { "id": "rel3", "followee": { "id": "u3", "username": "carol" } },
            { "id": "rel4", "followee": { "id": "u4", "username": "dave" } }
        ]);
        let two_notifications = json!([
            {
                "id": "notif1",
                "type": "follow",
                "createdAt": "2026-11-12T03:00:00.000Z",
                "user": { "id": "u1", "username": "alice" }
            },
            {
                "id": "notif2",
                "type": "follow",
                "createdAt": "2026-11-12T03:00:00.000Z",
                "user": { "id": "u2", "username": "bob" }
            }
        ]);
        let (client, _) = instance_with(vec![
            ok_json(&two_notes),
            ok_json(&two_notes),
            ok_json(&two_followers),
            ok_json(&two_followers),
            ok_json(&two_following),
            ok_json(&two_notifications),
            ok_json(&two_notes),
        ]);

        // Full page (2 == limit 2) -> has cursor
        let full_tl =
            block_on(client.timeline(StreamKind::Local, PageParams::first(2))).expect("tl");
        assert_eq!(full_tl.next_max_id.as_deref(), Some("n2"));

        // Short page (2 < limit 10) -> no cursor
        let short_tl =
            block_on(client.timeline(StreamKind::Local, PageParams::first(10))).expect("tl");
        assert_eq!(short_tl.next_max_id, None);

        // Followers: full page -> cursor, short page -> None
        let full_followers =
            block_on(client.account_followers(&LocalId("u0".to_owned()), PageParams::first(2)))
                .expect("followers");
        assert_eq!(full_followers.next_max_id.as_deref(), Some("rel2"));

        let short_followers =
            block_on(client.account_followers(&LocalId("u0".to_owned()), PageParams::first(5)))
                .expect("followers");
        assert_eq!(short_followers.next_max_id, None);

        // Following, notifications, account_statuses short pages -> None
        let short_following =
            block_on(client.account_following(&LocalId("u0".to_owned()), PageParams::first(5)))
                .expect("following");
        assert_eq!(short_following.next_max_id, None);

        let short_notifs =
            block_on(client.notifications(PageParams::first(5))).expect("notifications");
        assert_eq!(short_notifs.next_max_id, None);

        let short_statuses =
            block_on(client.account_statuses(&LocalId("u0".to_owned()), PageParams::first(5)))
                .expect("account_statuses");
        assert_eq!(short_statuses.next_max_id, None);
    }
}
