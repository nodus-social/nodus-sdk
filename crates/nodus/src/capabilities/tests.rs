//! Tests for capability parsing and baseline dispatch.

use super::*;

#[test]
fn a_plain_mastodon_instance_serves_its_baseline_but_no_extensions() {
    let capabilities = Capabilities::mastodon();
    assert_eq!(capabilities.provider(), Provider::Mastodon);
    assert!(!capabilities.knows_extensions());
    assert!(!capabilities.supports(Feature::Batch));
    assert!(!capabilities.supports_all(&[Feature::Events]));
}

#[test]
fn unknown_server_features_are_retained_rather_than_rejected() {
    let meta = serde_json::json!({
        "contract": "0.2.0",
        "features": ["batch", "telepathy", "events"]
    });
    let capabilities = Capabilities::from_meta(&meta).expect("meta");
    assert_eq!(capabilities.contract, "0.2.0");
    assert_eq!(capabilities.features, vec![Feature::Batch, Feature::Events]);
    assert_eq!(capabilities.unknown, vec!["telepathy".to_owned()]);
    assert!(capabilities.supports(Feature::Batch));
    assert!(!capabilities.supports(Feature::SyncToken));
}

#[test]
fn a_typed_meta_answer_keeps_unknown_names_too() {
    let capabilities =
        Capabilities::from_parts("0.1.0", &["polls".to_owned(), "quantum".to_owned()]);
    assert_eq!(capabilities.features, vec![Feature::Polls]);
    assert_eq!(capabilities.unknown, vec!["quantum".to_owned()]);
}

#[test]
fn a_meta_document_without_a_feature_list_means_no_extensions() {
    let capabilities =
        Capabilities::from_meta(&serde_json::json!({ "contract": "0.1.0" })).expect("meta");
    assert_eq!(capabilities.features, vec![]);
    assert_eq!(capabilities.provider, Provider::Nodus);
}

#[test]
fn a_non_object_meta_document_is_an_error() {
    let error = Capabilities::from_meta(&serde_json::json!(["batch"])).expect_err("array");
    assert_eq!(error, MetaError::NotAnObject);
}

#[test]
fn the_provider_defaults_to_mastodon_when_an_older_answer_is_deserialised() {
    let capabilities: Capabilities =
        serde_json::from_value(serde_json::json!({ "contract": "" })).expect("capabilities");
    assert_eq!(capabilities.provider, Provider::Mastodon);
    assert_eq!(capabilities.unknown, Vec::<String>::new());
}
