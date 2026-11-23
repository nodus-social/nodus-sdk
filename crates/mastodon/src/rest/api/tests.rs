use core::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use serde_json::Value;

use crate::actions::{
    AccountActions, AdminActions, Browse, FilterActions, FilterV1Actions, HighlightActions,
    InstanceMeta, MarkerActions, ScheduleActions, SessionActions, SettingsActions, StatusActions,
    TagActions,
};
use crate::api::InstanceApi;
use crate::model::{
    Draft, FilterAction, FilterContext, FilterDraft, FilterV1Draft, LocalId, MarkerTimeline,
    MarkerUpdate, PollDraft, PreferenceRefusal, Preferences, PreferencesPatch, ProfileField,
    ProfileImage, ProfileUpdate, Visibility,
};
use crate::rest::{HttpRequest, HttpResponse, RestInstance, Transport};
use crate::{Error, Instance, Result};

#[derive(Clone)]
struct RecordedRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

impl RecordedRequest {
    fn form_pairs(&self) -> Vec<(String, String)> {
        let bytes = self.body.as_deref().unwrap_or_default();
        url::form_urlencoded::parse(bytes)
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
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
        self.requests.borrow_mut().push(RecordedRequest {
            method: request.method.as_str().to_owned(),
            url: request.url.to_string(),
            headers: request.headers,
            body: request.body,
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
    RestInstance<MockTransport>,
    Rc<RefCell<Vec<RecordedRequest>>>,
) {
    let transport = MockTransport::with_responses(responses);
    let requests = Rc::clone(&transport.requests);
    let instance = Instance::parse("https://mastodon.example").expect("valid host");
    (
        RestInstance::with_transport(&instance, "token", transport),
        requests,
    )
}

fn ok_json(json: &str) -> HttpResponse {
    HttpResponse {
        status: 200,
        retry_after_s: None,
        body: json.as_bytes().to_vec(),
    }
}

fn ok_empty(status: u16) -> HttpResponse {
    HttpResponse {
        status,
        retry_after_s: None,
        body: Vec::new(),
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

const MINIMAL_STATUS_JSON: &str = r#"{"id":"1","created_at":"2026-01-01T00:00:00Z","content":"","visibility":"public","account":{"id":"1","acct":"alice","display_name":"Alice"}}"#;
const MINIMAL_ACCOUNT_JSON: &str = r#"{"id":"1","acct":"alice","display_name":"Alice"}"#;
const MINIMAL_FILTER_JSON: &str =
    r#"{"id":"1","title":"Spoilers","context":["home"],"filter_action":"warn","keywords":[]}"#;
const MINIMAL_FILTER_V1_JSON: &str =
    r#"{"id":"1","phrase":"spoiler","context":["home"],"irreversible":true}"#;
const MINIMAL_SCHEDULED_JSON: &str =
    r#"{"id":"1","scheduled_at":"2026-12-01T12:00:00Z","params":{"text":"hello"}}"#;

#[test]
fn vote_decodes_poll_directly_from_votes_endpoint() {
    let poll_json = r#"{
        "id": "42",
        "expires_at": null,
        "expired": false,
        "multiple": true,
        "votes_count": 3,
        "voters_count": 2,
        "voted": true,
        "own_votes": [0, 2],
        "options": [
            {"title": "a", "votes_count": 2},
            {"title": "b", "votes_count": 1}
        ],
        "emojis": []
    }"#;
    let (instance, requests) = instance_with(vec![ok_json(poll_json)]);

    let poll: crate::model::Poll = block_on(instance.vote("42", &[0, 2]))
        .expect("vote should decode Poll from POST /api/v1/polls/42/votes");
    assert_eq!(poll.id, "42");
    assert!(poll.voted);
    assert_eq!(poll.own_votes, vec![0, 2]);
    assert_eq!(poll.votes_count, 3);

    let recorded = requests.borrow();
    assert_eq!(recorded.len(), 1, "vote should only issue one POST request");
    assert_eq!(recorded[0].method, "POST");
    assert_eq!(
        recorded[0].url,
        "https://mastodon.example/api/v1/polls/42/votes"
    );
    assert_eq!(
        recorded[0].form_pairs(),
        vec![
            ("choices[]".to_owned(), "0".to_owned()),
            ("choices[]".to_owned(), "2".to_owned()),
        ]
    );
}

#[test]
fn unit_endpoints_succeed_on_empty_bodies_and_json_objects() {
    for response in [
        ok_empty(200),
        ok_empty(204),
        ok_json("{}"),
        ok_json(r#"{"id":"9","following":false}"#),
    ] {
        let (instance, _) = instance_with(vec![
            response.clone(),
            response.clone(),
            response.clone(),
            response.clone(),
            response.clone(),
            response.clone(),
            response.clone(),
            response.clone(),
            response.clone(),
            response.clone(),
            response.clone(),
            response,
        ]);

        let id = LocalId("9".to_owned());
        block_on(instance.reject_follow_request(&id)).expect("reject_follow_request");
        block_on(instance.delete_domain_allow("9")).expect("delete_domain_allow");
        block_on(instance.delete_domain_block("9")).expect("delete_domain_block");
        block_on(instance.delete_filter("9")).expect("delete_filter");
        block_on(instance.unfeature_tag("9")).expect("unfeature_tag");
        block_on(instance.cancel_scheduled("9")).expect("cancel_scheduled");
        block_on(instance.dismiss_announcement("9")).expect("dismiss_announcement");
        block_on(instance.delete_status("9")).expect("delete_status");
        block_on(instance.delete_list("9")).expect("delete_list");
        block_on(instance.add_to_list("9", std::slice::from_ref(&id))).expect("add_to_list");
        block_on(instance.remove_from_list("9", std::slice::from_ref(&id)))
            .expect("remove_from_list");
        block_on(instance.mark_conversation_read("9")).expect("mark_conversation_read");
    }
}

#[test]
fn unit_endpoints_map_non_2xx_statuses_to_errors() {
    let (instance, _) = instance_with(vec![
        HttpResponse {
            status: 401,
            retry_after_s: None,
            body: Vec::new(),
        },
        HttpResponse {
            status: 429,
            retry_after_s: Some(60),
            body: Vec::new(),
        },
        HttpResponse {
            status: 422,
            retry_after_s: None,
            body: br#"{"error":"invalid"}"#.to_vec(),
        },
    ]);

    assert!(matches!(
        block_on(instance.delete_filter("1")),
        Err(Error::Unauthorized)
    ));
    assert!(matches!(
        block_on(instance.delete_filter("1")),
        Err(Error::RateLimited { retry_after_s: 60 })
    ));
    assert!(matches!(
        block_on(instance.delete_filter("1")),
        Err(Error::Status {
            status: 422,
            message
        }) if message == "invalid"
    ));
}

#[test]
fn filter_actions_send_whole_word_per_keyword_not_top_level() {
    let (instance, requests) = instance_with(vec![
        ok_json(MINIMAL_FILTER_JSON),
        ok_json(MINIMAL_FILTER_JSON),
    ]);
    let draft = FilterDraft::new("Spoilers")
        .contexts(vec![FilterContext::Home])
        .action(FilterAction::Warn)
        .keywords(vec!["cat".to_owned(), "dog".to_owned()])
        .whole_word(true);

    block_on(instance.create_filter(&draft)).expect("create_filter");
    block_on(instance.update_filter("7", &draft)).expect("update_filter");

    let recorded = requests.borrow();
    assert_eq!(recorded.len(), 2);
    for req in recorded.iter() {
        let pairs = req.form_pairs();
        assert!(
            !pairs.iter().any(|(k, _)| k == "whole_word"),
            "top-level whole_word must not be sent: {pairs:?}"
        );
        let keyword_attrs: Vec<(&str, &str)> = pairs
            .iter()
            .filter(|(k, _)| k.starts_with("keywords_attributes"))
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            keyword_attrs,
            vec![
                ("keywords_attributes[][keyword]", "cat"),
                ("keywords_attributes[][whole_word]", "true"),
                ("keywords_attributes[][keyword]", "dog"),
                ("keywords_attributes[][whole_word]", "true"),
            ]
        );
    }
}

#[test]
fn update_profile_sends_bot_and_discoverable_fields() {
    let (instance, requests) = instance_with(vec![ok_json(MINIMAL_ACCOUNT_JSON)]);
    let update = ProfileUpdate {
        display_name: Some("Alice".to_owned()),
        note: Some("Hello".to_owned()),
        locked: Some(true),
        bot: Some(true),
        discoverable: Some(false),
        avatar: None,
        header: None,
        fields: None,
        avatar_image: None,
        header_image: None,
    };

    block_on(instance.update_profile(&update)).expect("update_profile");

    let recorded = requests.borrow();
    let pairs = recorded[0].form_pairs();
    assert!(pairs.contains(&("display_name".to_owned(), "Alice".to_owned())));
    assert!(pairs.contains(&("note".to_owned(), "Hello".to_owned())));
    assert!(pairs.contains(&("locked".to_owned(), "true".to_owned())));
    assert!(pairs.contains(&("bot".to_owned(), "true".to_owned())));
    assert!(pairs.contains(&("discoverable".to_owned(), "false".to_owned())));
}

#[test]
fn update_profile_sends_uploaded_pictures_by_id() {
    let (instance, requests) = instance_with(vec![ok_json(MINIMAL_ACCOUNT_JSON)]);
    let update = ProfileUpdate::new().avatar("media-1").header("media-2");

    block_on(instance.update_profile(&update)).expect("update_profile");

    let recorded = requests.borrow();
    let pairs = recorded[0].form_pairs();
    assert!(pairs.contains(&("avatar".to_owned(), "media-1".to_owned())));
    assert!(pairs.contains(&("header".to_owned(), "media-2".to_owned())));
    assert!(!pairs.iter().any(|(key, _)| key == "display_name"));
}

#[test]
fn update_profile_without_fields_keeps_the_form_body() {
    let (instance, requests) = instance_with(vec![ok_json(MINIMAL_ACCOUNT_JSON)]);
    let update = ProfileUpdate::new().display_name("Alice").note("Hello");

    block_on(instance.update_profile(&update)).expect("update_profile");

    let recorded = requests.borrow();
    let request = &recorded[0];
    assert_eq!(
        request.header("content-type"),
        Some("application/x-www-form-urlencoded")
    );
    assert_eq!(
        request.form_pairs(),
        vec![
            ("display_name".to_owned(), "Alice".to_owned()),
            ("note".to_owned(), "Hello".to_owned()),
        ]
    );
}

#[test]
fn update_profile_sends_an_empty_field_list_as_json() {
    let (instance, requests) = instance_with(vec![ok_json(MINIMAL_ACCOUNT_JSON)]);
    let update = ProfileUpdate::new()
        .display_name("Alice")
        .fields(Vec::new());

    block_on(instance.update_profile(&update)).expect("update_profile");

    let recorded = requests.borrow();
    let request = &recorded[0];
    assert_eq!(request.header("content-type"), Some("application/json"));
    let body: Value =
        serde_json::from_slice(request.body.as_deref().unwrap_or_default()).expect("json body");
    assert_eq!(body["display_name"], "Alice");
    assert_eq!(body["fields"], serde_json::json!([]));
    assert_eq!(body["fields_attributes"], serde_json::json!([]));
    assert!(body.get("note").is_none());
}

#[test]
fn update_profile_sends_six_metadata_rows_in_order() {
    let (instance, requests) = instance_with(vec![ok_json(MINIMAL_ACCOUNT_JSON)]);
    let fields: Vec<ProfileField> = (1..=6)
        .map(|row| ProfileField::new(format!("Row {row}"), format!("value {row}")))
        .collect();
    let update = ProfileUpdate::new().fields(fields);

    block_on(instance.update_profile(&update)).expect("update_profile");

    let recorded = requests.borrow();
    let body: Value =
        serde_json::from_slice(recorded[0].body.as_deref().unwrap_or_default()).expect("json body");
    let rows = body["fields"].as_array().expect("fields array");
    assert_eq!(rows.len(), 6);
    assert_eq!(rows[0]["name"], "Row 1");
    assert_eq!(rows[5]["name"], "Row 6");
    assert_eq!(rows[5]["value"], "value 6");
    assert_eq!(body["fields_attributes"], body["fields"]);
}

#[test]
fn edit_status_sends_all_draft_fields_and_clears_cw_and_sensitive() {
    let (instance, requests) = instance_with(vec![
        ok_json(MINIMAL_STATUS_JSON),
        ok_json(MINIMAL_STATUS_JSON),
    ]);

    let full_draft = Draft {
        text: "edited".to_owned(),
        visibility: Visibility::Unlisted,
        in_reply_to_id: None,
        idempotency_key: None,
        media_ids: vec!["m1".to_owned(), "m2".to_owned()],
        spoiler_text: "cw".to_owned(),
        sensitive: true,
        poll: Some(
            PollDraft::new(vec!["yes".to_owned(), "no".to_owned()], 3600)
                .multiple(true)
                .hide_totals(true),
        ),
        language: Some("de".to_owned()),
    };
    block_on(instance.edit_status("10", &full_draft)).expect("edit_status full");

    let clear_draft = Draft::text("cleared", Visibility::Public);
    block_on(instance.edit_status("10", &clear_draft)).expect("edit_status clear");

    let recorded = requests.borrow();
    let full_pairs = recorded[0].form_pairs();
    assert!(full_pairs.contains(&("media_ids[]".to_owned(), "m1".to_owned())));
    assert!(full_pairs.contains(&("media_ids[]".to_owned(), "m2".to_owned())));
    assert!(full_pairs.contains(&("language".to_owned(), "de".to_owned())));
    assert!(full_pairs.contains(&("spoiler_text".to_owned(), "cw".to_owned())));
    assert!(full_pairs.contains(&("sensitive".to_owned(), "true".to_owned())));
    assert!(full_pairs.contains(&("poll[options][]".to_owned(), "yes".to_owned())));
    assert!(full_pairs.contains(&("poll[options][]".to_owned(), "no".to_owned())));
    assert!(full_pairs.contains(&("poll[expires_in]".to_owned(), "3600".to_owned())));
    assert!(full_pairs.contains(&("poll[multiple]".to_owned(), "true".to_owned())));
    assert!(full_pairs.contains(&("poll[hide_totals]".to_owned(), "true".to_owned())));

    let clear_pairs = recorded[1].form_pairs();
    assert!(
        clear_pairs.contains(&("spoiler_text".to_owned(), String::new())),
        "clearing CW must send empty spoiler_text: {clear_pairs:?}"
    );
    assert!(
        clear_pairs.contains(&("sensitive".to_owned(), "false".to_owned())),
        "unmarking sensitive must send sensitive=false: {clear_pairs:?}"
    );
}

#[test]
fn schedule_sends_language_poll_and_idempotency_key() {
    let (instance, requests) = instance_with(vec![ok_json(MINIMAL_SCHEDULED_JSON)]);
    let draft = Draft {
        text: "later".to_owned(),
        visibility: Visibility::Public,
        in_reply_to_id: Some("99".to_owned()),
        idempotency_key: Some("idem-key-1".to_owned()),
        media_ids: vec!["m1".to_owned()],
        spoiler_text: "cw".to_owned(),
        sensitive: true,
        poll: Some(
            PollDraft::new(vec!["one".to_owned(), "two".to_owned()], 600)
                .multiple(true)
                .hide_totals(true),
        ),
        language: Some("fr".to_owned()),
    };

    block_on(instance.schedule(&draft, "2026-12-01T12:00:00Z")).expect("schedule");

    let recorded = requests.borrow();
    let pairs = recorded[0].form_pairs();
    assert!(pairs.contains(&("in_reply_to_id".to_owned(), "99".to_owned())));
    assert!(pairs.contains(&("media_ids[]".to_owned(), "m1".to_owned())));
    assert!(pairs.contains(&("spoiler_text".to_owned(), "cw".to_owned())));
    assert!(pairs.contains(&("sensitive".to_owned(), "true".to_owned())));
    assert!(pairs.contains(&("language".to_owned(), "fr".to_owned())));
    assert!(pairs.contains(&("poll[options][]".to_owned(), "one".to_owned())));
    assert!(pairs.contains(&("poll[options][]".to_owned(), "two".to_owned())));
    assert!(pairs.contains(&("poll[expires_in]".to_owned(), "600".to_owned())));
    assert!(pairs.contains(&("poll[multiple]".to_owned(), "true".to_owned())));
    assert!(pairs.contains(&("poll[hide_totals]".to_owned(), "true".to_owned())));
    assert!(pairs.contains(&("scheduled_at".to_owned(), "2026-12-01T12:00:00Z".to_owned())));
    assert_eq!(recorded[0].header("Idempotency-Key"), Some("idem-key-1"));
}

#[test]
fn post_status_sends_draft_fields_and_idempotency_key() {
    let (instance, requests) = instance_with(vec![ok_json(MINIMAL_STATUS_JSON)]);
    let draft = Draft {
        text: "hello".to_owned(),
        visibility: Visibility::Public,
        in_reply_to_id: Some("12".to_owned()),
        idempotency_key: Some("post-idem-1".to_owned()),
        media_ids: vec!["m1".to_owned()],
        spoiler_text: String::new(),
        sensitive: false,
        poll: None,
        language: Some("en".to_owned()),
    };

    block_on(instance.post_status(&draft)).expect("post_status");

    let recorded = requests.borrow();
    let pairs = recorded[0].form_pairs();
    assert!(pairs.contains(&("status".to_owned(), "hello".to_owned())));
    assert!(pairs.contains(&("in_reply_to_id".to_owned(), "12".to_owned())));
    assert!(pairs.contains(&("media_ids[]".to_owned(), "m1".to_owned())));
    assert!(pairs.contains(&("language".to_owned(), "en".to_owned())));
    assert!(!pairs.iter().any(|(k, _)| k == "spoiler_text"));
    assert!(!pairs.iter().any(|(k, _)| k == "sensitive"));
    assert_eq!(recorded[0].header("Idempotency-Key"), Some("post-idem-1"));
}

#[test]
fn markers_read_asks_only_for_the_requested_timelines() {
    let (instance, requests) = instance_with(vec![ok_json(
        r#"{"home":{"last_read_id":"42","version":3},"notifications":{"last_read_id":"9"}}"#,
    )]);

    let markers = block_on(nodus_client_core::actions::MarkerActions::markers(
        &instance,
        &[MarkerTimeline::Notifications],
    ))
    .expect("markers");
    assert_eq!(
        markers
            .get(MarkerTimeline::Notifications)
            .map(|marker| marker.last_read_id.as_str()),
        Some("9")
    );

    let recorded = requests.borrow();
    assert_eq!(recorded[0].method, "GET");
    assert_eq!(
        recorded[0].url,
        "https://mastodon.example/api/v1/markers?timeline%5B%5D=notifications"
    );
}

#[test]
fn markers_write_names_each_timeline_in_the_form() {
    let (instance, requests) = instance_with(vec![ok_json(
        r#"{"home":{"last_read_id":"42","version":4}}"#,
    )]);

    let updates = [
        MarkerUpdate::new(MarkerTimeline::Home, "42").version(4),
        MarkerUpdate::new(MarkerTimeline::Notifications, "9"),
    ];
    block_on(instance.save_markers(&updates)).expect("save_markers");

    let recorded = requests.borrow();
    assert_eq!(recorded[0].method, "POST");
    assert_eq!(recorded[0].url, "https://mastodon.example/api/v1/markers");
    assert_eq!(
        recorded[0].form_pairs(),
        vec![
            ("home[last_read_id]".to_owned(), "42".to_owned()),
            ("home[version]".to_owned(), "4".to_owned()),
            ("notifications[last_read_id]".to_owned(), "9".to_owned()),
        ]
    );
}

#[test]
fn tag_endpoints_address_the_tag_by_name_and_encode_it() {
    let (instance, requests) = instance_with(vec![
        ok_json("[]"),
        ok_json(r#"{"name":"rust","following":false}"#),
        ok_json(r#"{"name":"rust","following":true}"#),
        ok_json(r#"{"name":"rust","following":false}"#),
    ]);

    block_on(instance.own_featured_tags()).expect("own_featured_tags");
    block_on(instance.tag("#rust")).expect("tag");
    block_on(instance.follow_tag("#rust")).expect("follow_tag");
    block_on(instance.unfollow_tag("#rust")).expect("unfollow_tag");

    let recorded = requests.borrow();
    assert_eq!(recorded[0].method, "GET");
    assert_eq!(
        recorded[0].url,
        "https://mastodon.example/api/v1/featured_tags"
    );
    assert_eq!(
        recorded[1].url,
        "https://mastodon.example/api/v1/tags/%23rust"
    );
    assert_eq!(
        recorded[2].url,
        "https://mastodon.example/api/v1/tags/%23rust/follow"
    );
    assert_eq!(recorded[2].method, "POST");
    assert_eq!(
        recorded[3].url,
        "https://mastodon.example/api/v1/tags/%23rust/unfollow"
    );
}

#[test]
fn filter_v1_writes_every_context_as_a_repeated_field() {
    let (instance, requests) = instance_with(vec![
        ok_json("[]"),
        ok_json(MINIMAL_FILTER_V1_JSON),
        ok_json(MINIMAL_FILTER_V1_JSON),
        ok_empty(200),
    ]);
    let draft = FilterV1Draft::new("spoiler")
        .contexts(vec![FilterContext::Home, FilterContext::Public])
        .action(FilterAction::Hide)
        .whole_word(true);

    block_on(instance.filters_v1()).expect("filters_v1");
    block_on(instance.create_filter_v1(&draft)).expect("create_filter_v1");
    block_on(instance.update_filter_v1("7", &draft)).expect("update_filter_v1");
    block_on(instance.delete_filter_v1("7")).expect("delete_filter_v1");

    let recorded = requests.borrow();
    assert_eq!(recorded[0].url, "https://mastodon.example/api/v1/filters");
    assert_eq!(recorded[1].method, "POST");
    assert_eq!(recorded[1].url, "https://mastodon.example/api/v1/filters");
    assert_eq!(recorded[1].form_pairs(), expected_v1_form());
    assert_eq!(recorded[2].method, "PUT");
    assert_eq!(recorded[2].url, "https://mastodon.example/api/v1/filters/7");
    assert_eq!(recorded[2].form_pairs(), expected_v1_form());
    assert_eq!(recorded[3].method, "DELETE");
    assert_eq!(recorded[3].url, "https://mastodon.example/api/v1/filters/7");
}

fn expected_v1_form() -> Vec<(String, String)> {
    vec![
        ("phrase".to_owned(), "spoiler".to_owned()),
        ("context[]".to_owned(), "home".to_owned()),
        ("context[]".to_owned(), "public".to_owned()),
        ("irreversible".to_owned(), "true".to_owned()),
        ("whole_word".to_owned(), "true".to_owned()),
    ]
}

#[test]
fn update_profile_uploads_images_as_multipart_parts() {
    let (instance, requests) = instance_with(vec![ok_json(MINIMAL_ACCOUNT_JSON)]);
    let update = ProfileUpdate {
        display_name: Some("Alice".to_owned()),
        note: None,
        locked: None,
        bot: None,
        discoverable: None,
        avatar: None,
        header: None,
        fields: None,
        avatar_image: Some(ProfileImage::new(b"avatar-bytes", "image/png")),
        header_image: None,
    };

    block_on(instance.update_profile(&update)).expect("update_profile");

    let recorded = requests.borrow();
    let request = &recorded[0];
    assert_eq!(request.method, "PATCH");
    let content_type = request.header("content-type").expect("content type");
    assert!(content_type.starts_with("multipart/form-data; boundary="));
    let body = String::from_utf8_lossy(request.body.as_deref().unwrap_or_default());
    assert!(body.contains("name=\"display_name\""));
    assert!(body.contains("name=\"avatar\"; filename=\"avatar\""));
    assert!(body.contains("Content-Type: image/png"));
    assert!(body.contains("avatar-bytes"));
}

#[test]
fn discovery_endpoints_carry_their_limit() {
    let (instance, requests) = instance_with(vec![
        ok_json(&format!(
            r#"[{{"account":{MINIMAL_ACCOUNT_JSON},"source":"staff"}}]"#
        )),
        ok_json(r#"[{"url":"https://example.com/a","title":"A","type":"link"}]"#),
    ]);

    let suggestions = block_on(instance.suggestions(5)).expect("suggestions");
    assert_eq!(suggestions.len(), 1);
    let links = block_on(instance.trending_links(3)).expect("trending_links");
    assert_eq!(links.len(), 1);

    let recorded = requests.borrow();
    assert_eq!(
        recorded[0].url,
        "https://mastodon.example/api/v2/suggestions?limit=5"
    );
    assert_eq!(
        recorded[1].url,
        "https://mastodon.example/api/v1/trends/links?limit=3"
    );
}

#[test]
fn parity_endpoints_map_a_rejected_write_to_the_server_message() {
    let rejected = || HttpResponse {
        status: 422,
        retry_after_s: None,
        body: br#"{"error":"invalid"}"#.to_vec(),
    };
    let (instance, _) = instance_with(vec![
        rejected(),
        rejected(),
        rejected(),
        rejected(),
        rejected(),
        rejected(),
        rejected(),
        rejected(),
        rejected(),
        rejected(),
        rejected(),
        rejected(),
        rejected(),
    ]);
    let draft = FilterV1Draft::new("spoiler");
    let updates = [MarkerUpdate::new(MarkerTimeline::Home, "42")];
    let profile = ProfileUpdate {
        display_name: Some("Alice".to_owned()),
        ..ProfileUpdate::default()
    };

    let rejected = |result: Result<()>| {
        matches!(
            result,
            Err(Error::Status {
                status: 422,
                ref message
            }) if message == "invalid"
        )
    };
    assert!(rejected(
        block_on(nodus_client_core::actions::MarkerActions::markers(
            &instance,
            &[MarkerTimeline::Home]
        ))
        .map(|_| ())
    ));
    assert!(rejected(
        block_on(instance.save_markers(&updates)).map(|_| ())
    ));
    assert!(rejected(block_on(instance.own_featured_tags()).map(|_| ())));
    assert!(rejected(block_on(instance.tag("rust")).map(|_| ())));
    assert!(rejected(block_on(instance.follow_tag("rust")).map(|_| ())));
    assert!(rejected(
        block_on(instance.unfollow_tag("rust")).map(|_| ())
    ));
    assert!(rejected(block_on(instance.filters_v1()).map(|_| ())));
    assert!(rejected(
        block_on(instance.create_filter_v1(&draft)).map(|_| ())
    ));
    assert!(rejected(
        block_on(instance.update_filter_v1("7", &draft)).map(|_| ())
    ));
    assert!(rejected(block_on(instance.delete_filter_v1("7"))));
    assert!(rejected(
        block_on(instance.update_profile(&profile)).map(|_| ())
    ));
    assert!(rejected(block_on(instance.suggestions(5)).map(|_| ())));
    assert!(rejected(block_on(instance.trending_links(3)).map(|_| ())));
}

const PREFERENCES_JSON: &str = r#"{"posting:default:visibility":"public","posting:default:sensitive":true,"posting:default:language":null,"posting:default:quote_policy":"public","reading:expand:media":"default","reading:expand:spoilers":false,"reading:autoplay:gifs":false,"notification:email":true,"notification:report":true}"#;

#[test]
fn update_preferences_puts_only_the_changed_keys_and_decodes_the_whole_map() {
    let (instance, requests) = instance_with(vec![ok_json(PREFERENCES_JSON)]);
    let current = Preferences::default();
    let mut desired = current.clone();
    desired.posting_default_sensitive = true;
    let patch = PreferencesPatch::between(&current, &desired);

    let preferences = block_on(SettingsActions::update_preferences(&instance, &patch))
        .expect("update_preferences");
    assert!(preferences.posting_default_sensitive);
    assert_eq!(preferences.posting_default_visibility, Visibility::Public);

    let recorded = requests.borrow();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].method, "PUT");
    assert_eq!(
        recorded[0].url,
        "https://mastodon.example/api/v1/preferences"
    );
    let body: Value =
        serde_json::from_slice(recorded[0].body.as_deref().unwrap_or_default()).expect("JSON body");
    assert_eq!(body, serde_json::json!({"posting:default:sensitive": true}));
}

#[test]
fn update_preferences_maps_a_refused_key_to_a_typed_error() {
    let body = br#"{"error":"unknown preference: posting:default:typo"}"#.to_vec();
    let (instance, _) = instance_with(vec![HttpResponse {
        status: 422,
        retry_after_s: None,
        body,
    }]);
    let patch = PreferencesPatch {
        posting_default_sensitive: Some(true),
        ..PreferencesPatch::default()
    };

    let error = block_on(SettingsActions::update_preferences(&instance, &patch))
        .expect_err("the server refused the key");
    assert!(matches!(
        error,
        Error::PreferenceRefused(PreferenceRefusal::UnknownKey { key })
            if key == "posting:default:typo"
    ));
}

#[test]
fn update_preferences_maps_a_refused_value_to_key_and_allowed_values() {
    let body = br#"{"error":"invalid value for posting:default:visibility: sideways; allowed values are public, unlisted, private and direct"}"#.to_vec();
    let (instance, _) = instance_with(vec![HttpResponse {
        status: 422,
        retry_after_s: None,
        body,
    }]);
    let patch = PreferencesPatch {
        posting_default_visibility: Some(Visibility::Public),
        ..PreferencesPatch::default()
    };

    let error = block_on(SettingsActions::update_preferences(&instance, &patch))
        .expect_err("the server refused the value");
    assert!(matches!(
        error,
        Error::PreferenceRefused(PreferenceRefusal::InvalidValue {
            key,
            value,
            allowed,
        }) if key == "posting:default:visibility"
            && value == "sideways"
            && allowed == "public, unlisted, private and direct"
    ));
}

#[test]
fn sessions_reads_the_list_and_revokes_one_or_a_scope() {
    let sessions = ok_json(
        r#"[{"id":"7","created_at":"2026-01-01T00:00:00Z","last_used_at":null,"user_agent":"Firefox","ip":"203.0.113.0/24","current":true}]"#,
    );
    let (instance, requests) =
        instance_with(vec![sessions, ok_empty(204), ok_empty(204), ok_empty(204)]);

    let list = block_on(SessionActions::sessions(&instance)).expect("sessions");
    assert_eq!(list.len(), 1);
    assert!(list[0].current);
    assert_eq!(list[0].ip.as_deref(), Some("203.0.113.0/24"));

    block_on(SessionActions::revoke_session(&instance, "9")).expect("revoke one");
    block_on(SessionActions::revoke_other_sessions(&instance)).expect("revoke others");
    block_on(SessionActions::revoke_all_sessions(&instance)).expect("revoke all");

    let recorded = requests.borrow();
    assert_eq!(recorded.len(), 4);
    assert_eq!(recorded[0].method, "GET");
    assert_eq!(
        recorded[0].url,
        "https://mastodon.example/api/v1/auth/sessions"
    );
    assert_eq!(
        recorded[1].url,
        "https://mastodon.example/api/v1/auth/sessions/9"
    );
    assert_eq!(recorded[1].method, "DELETE");
    assert_eq!(
        recorded[2].url,
        "https://mastodon.example/api/v1/auth/sessions?scope=others"
    );
    assert_eq!(
        recorded[3].url,
        "https://mastodon.example/api/v1/auth/sessions?scope=all"
    );
}
