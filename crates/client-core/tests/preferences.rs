//! The typed preferences contract: the server's exact body, the partial encoder,
//! and the refusal parser.

use nodus_client_core::model::{
    ExpandMedia, PreferenceRefusal, Preferences, PreferencesPatch, QuotePolicy, Visibility,
};

const SERVER_JSON: &str = r#"{"posting:default:visibility":"public","posting:default:sensitive":false,"posting:default:language":null,"posting:default:quote_policy":"public","reading:expand:media":"default","reading:expand:spoilers":false,"reading:autoplay:gifs":false,"notification:email":true,"notification:report":true}"#;

#[test]
fn the_server_body_round_trips_and_equals_the_defaults() {
    let preferences: Preferences = serde_json::from_str(SERVER_JSON).expect("decode server body");
    assert_eq!(preferences, Preferences::default());
    assert_eq!(preferences.posting_default_visibility, Visibility::Public);
    assert_eq!(
        preferences.posting_default_quote_policy,
        QuotePolicy::Public
    );
    assert_eq!(preferences.reading_expand_media, ExpandMedia::Default);

    let encoded = serde_json::to_value(&preferences).expect("encode preferences");
    let expected: serde_json::Value = serde_json::from_str(SERVER_JSON).expect("server body");
    assert_eq!(encoded, expected);
}

#[test]
fn an_unknown_enum_value_reads_as_the_server_default_instead_of_failing() {
    let body = r#"{"posting:default:visibility":"sideways","posting:default:sensitive":false,"posting:default:language":null,"posting:default:quote_policy":"sideways","reading:expand:media":"sideways","reading:expand:spoilers":false,"reading:autoplay:gifs":false,"notification:email":false,"notification:report":false}"#;

    let preferences: Preferences =
        serde_json::from_str(body).expect("unknown values must not fail the read");
    assert_eq!(preferences.posting_default_visibility, Visibility::Public);
    assert_eq!(
        preferences.posting_default_quote_policy,
        QuotePolicy::Public
    );
    assert_eq!(preferences.reading_expand_media, ExpandMedia::Default);
}

#[test]
fn the_patch_encodes_only_the_keys_that_differ() {
    let current = Preferences::default();
    let mut desired = current.clone();
    desired.posting_default_sensitive = true;
    desired.posting_default_language = Some("en".to_owned());
    desired.reading_expand_media = ExpandMedia::HideAll;

    let patch = PreferencesPatch::between(&current, &desired);
    assert_eq!(
        serde_json::to_value(&patch).expect("encode patch"),
        serde_json::json!({
            "posting:default:sensitive": true,
            "posting:default:language": "en",
            "reading:expand:media": "hide_all",
        })
    );
}

#[test]
fn an_unchanged_patch_is_empty() {
    let current = Preferences::default();
    let patch = PreferencesPatch::between(&current, &current);
    assert!(patch.is_empty());
    assert_eq!(
        serde_json::to_value(&patch).expect("encode patch"),
        serde_json::json!({})
    );
}

#[test]
fn clearing_a_language_sends_explicit_null() {
    let current = Preferences {
        posting_default_language: Some("en".to_owned()),
        ..Preferences::default()
    };
    let mut desired = current.clone();
    desired.posting_default_language = None;

    let patch = PreferencesPatch::between(&current, &desired);
    assert_eq!(
        serde_json::to_value(&patch).expect("encode patch"),
        serde_json::json!({"posting:default:language": null})
    );
}

#[test]
fn enum_wire_spellings_are_exact_in_both_directions() {
    for (policy, wire) in [
        (QuotePolicy::Public, "public"),
        (QuotePolicy::Followers, "followers"),
        (QuotePolicy::Nobody, "nobody"),
    ] {
        assert_eq!(policy.to_string(), wire);
        assert_eq!(wire.parse::<QuotePolicy>().expect("parse policy"), policy);
    }

    for (media, wire) in [
        (ExpandMedia::Default, "default"),
        (ExpandMedia::ShowAll, "show_all"),
        (ExpandMedia::HideAll, "hide_all"),
    ] {
        assert_eq!(media.to_string(), wire);
        assert_eq!(wire.parse::<ExpandMedia>().expect("parse media"), media);
    }

    assert!("sideways".parse::<QuotePolicy>().is_err());
    assert!("sideways".parse::<ExpandMedia>().is_err());
}

#[test]
fn a_refusal_carries_the_key_and_allowed_values() {
    assert_eq!(
        PreferenceRefusal::parse("unknown preference: posting:default:typo"),
        Some(PreferenceRefusal::UnknownKey {
            key: "posting:default:typo".to_owned(),
        })
    );

    assert_eq!(
        PreferenceRefusal::parse(
            "invalid value for posting:default:visibility: sideways; allowed values are public, unlisted, private and direct"
        ),
        Some(PreferenceRefusal::InvalidValue {
            key: "posting:default:visibility".to_owned(),
            value: "sideways".to_owned(),
            allowed: "public, unlisted, private and direct".to_owned(),
        })
    );

    assert_eq!(PreferenceRefusal::parse("some other server trouble"), None);
}
