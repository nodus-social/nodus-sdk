//! Tests for the capability vocabulary itself.

use super::*;

#[test]
fn every_feature_spelling_round_trips() {
    for feature in Feature::ALL {
        assert_eq!(Feature::parse(feature.as_str()), Some(feature));
    }
}

#[test]
fn an_unknown_spelling_is_not_a_feature_rather_than_an_error() {
    assert_eq!(Feature::parse("telepathy"), None);
}

#[test]
fn indices_are_the_declaration_order_and_unique() {
    for (position, feature) in Feature::ALL.into_iter().enumerate() {
        assert_eq!(feature.index(), position, "{feature:?}");
    }
}

#[test]
fn the_names_nodus_publishes_all_parse() {
    let published = [
        "admin",
        "moderation",
        "scanners",
        "topology",
        "protocol",
        "remote_blocks",
        "polls",
        "batch",
        "appeals",
        "instance_rules",
        "moderation_rules",
        "media_quarantine",
        "block_lists",
        "moderation_stats",
        "remote_accounts",
        "warnings",
        "escalation",
        "federation_delivery",
        "domain_pauses",
        "settings_transfer",
        "report_forwarding",
        "account_deletion",
        "drive",
        "drive_share",
        "drive_attach",
        "feed_accounts",
        "events",
        "sync_token",
    ];
    for name in published {
        assert!(Feature::parse(name).is_some(), "{name} did not parse");
    }
}
