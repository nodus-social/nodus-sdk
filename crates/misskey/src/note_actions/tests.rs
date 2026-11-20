//! The wire shape of every verb, and the tally each one reports back.

use core::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use nodus_client_core::rest::{HttpRequest, HttpResponse, Transport};
use nodus_client_core::{Instance, Result};
use serde_json::{Value, json};

use crate::MisskeyInstance;

use super::Reactions;

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
                body: b"{}".to_vec(),
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

#[test]
fn favourite_posts_the_note_id_and_reads_the_note_field() {
    let response = json!({ "id": "n1", "isFavorited": false });
    let (client, requests) = instance_with(vec![ok_json(&response)]);

    let favourited = block_on(client.favourite("n1")).expect("favourite");

    let sent = &requests.borrow()[0];
    assert_eq!(
        sent.url,
        "https://misskey.example/api/notes/favorites/create"
    );
    assert_eq!(sent.body, json!({ "noteId": "n1" }));
    // The note's own field wins, even when it disagrees with the verb's intent.
    assert!(!favourited);
}

#[test]
fn unfavourite_deletes_by_note_id_and_trusts_the_verb_when_the_field_is_absent() {
    // The nodus backend serialises a note without `isFavorited`; the 2xx is then the
    // only thing that says the favourite is gone.
    let response = json!({ "id": "n1", "reactions": { "❤️": 1 } });
    let (client, requests) = instance_with(vec![ok_json(&response)]);

    let favourited = block_on(client.unfavourite("n1")).expect("unfavourite");

    let sent = &requests.borrow()[0];
    assert_eq!(
        sent.url,
        "https://misskey.example/api/notes/favorites/delete"
    );
    assert_eq!(sent.body, json!({ "noteId": "n1" }));
    assert!(!favourited);
}

#[test]
fn react_posts_the_reaction_then_reads_the_tally_back() {
    let note = json!({
        "id": "n1",
        "reactions": { "❤️": 2, ":blob:": 1 },
        "reactionCount": 3
    });
    let (client, requests) = instance_with(vec![ok_json(&json!({})), ok_json(&note)]);

    let reactions = block_on(client.react("n1", "❤️")).expect("react");

    let requests = requests.borrow();
    assert_eq!(
        requests[0].url,
        "https://misskey.example/api/notes/reactions/create"
    );
    assert_eq!(
        requests[0].body,
        json!({ "noteId": "n1", "reaction": "❤️" })
    );
    assert_eq!(requests[1].url, "https://misskey.example/api/notes/show");
    assert_eq!(requests[1].body, json!({ "noteId": "n1" }));
    assert_eq!(reactions.count("❤️"), 2);
    assert_eq!(reactions.total(), 3);
}

#[test]
fn unreact_deletes_by_note_id_alone_and_reads_the_remaining_tally() {
    let note = json!({ "id": "n1", "reactions": { ":blob:": 1 } });
    let (client, requests) = instance_with(vec![ok_json(&json!({})), ok_json(&note)]);

    let reactions = block_on(client.unreact("n1", "❤️")).expect("unreact");

    let requests = requests.borrow();
    assert_eq!(
        requests[0].url,
        "https://misskey.example/api/notes/reactions/delete"
    );
    assert_eq!(
        requests[0].body,
        json!({ "noteId": "n1" }),
        "Misskey removes the account's own reaction, so the emoji is not sent"
    );
    assert_eq!(requests[1].url, "https://misskey.example/api/notes/show");
    assert!(!reactions.contains("❤️"));
    assert_eq!(reactions.count(":blob:"), 1);
}

#[test]
fn reactions_are_read_from_the_note_field() {
    let note = json!({ "reactions": { "❤️": 2, ":blob:": 1 } });
    let reactions = Reactions::from_note(&note);

    assert_eq!(reactions.count("👍"), 0);
    assert_eq!(reactions.total(), 3);
    assert!(!reactions.is_empty());
    assert_eq!(
        reactions.iter().collect::<Vec<_>>(),
        vec![(":blob:", 1), ("❤️", 2)]
    );
    assert!(Reactions::from_note(&json!({})).is_empty());
}
