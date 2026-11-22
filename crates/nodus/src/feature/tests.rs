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
    // `nodus-backend/crates/api/src/meta.rs` publishes exactly these 42 names.
    let published = [
        "feed_accounts",
        "events",
        "sync_token",
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
        "favourites",
        "bookmarks",
        "conversations",
        "lists",
        "filters",
        "scheduled_statuses",
        "notifications",
        "announcements",
        "search",
        "directory",
        "markers",
        "preferences",
        "streaming",
        "tags",
    ];
    assert_eq!(published.len(), Feature::COUNT);
    for name in published {
        assert!(Feature::parse(name).is_some(), "{name} did not parse");
    }
}

#[test]
fn the_vocabulary_covers_the_published_list_exactly() {
    let mut ours: Vec<&str> = Feature::ALL
        .iter()
        .map(|feature| feature.as_str())
        .collect();
    let mut backend = vec![
        "feed_accounts",
        "events",
        "sync_token",
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
        "favourites",
        "bookmarks",
        "conversations",
        "lists",
        "filters",
        "scheduled_statuses",
        "notifications",
        "announcements",
        "search",
        "directory",
        "markers",
        "preferences",
        "streaming",
        "tags",
    ];
    ours.sort_unstable();
    backend.sort_unstable();
    assert_eq!(ours, backend);
}

#[test]
fn every_new_collection_spelling_is_its_own_wire_word() {
    for (feature, wire) in [
        (Feature::Bookmarks, "bookmarks"),
        (Feature::Conversations, "conversations"),
        (Feature::Lists, "lists"),
        (Feature::Filters, "filters"),
        (Feature::ScheduledStatuses, "scheduled_statuses"),
        (Feature::Notifications, "notifications"),
        (Feature::Announcements, "announcements"),
        (Feature::Search, "search"),
        (Feature::Directory, "directory"),
        (Feature::Markers, "markers"),
        (Feature::Preferences, "preferences"),
        (Feature::Streaming, "streaming"),
        (Feature::Tags, "tags"),
    ] {
        assert_eq!(feature.as_str(), wire);
        assert_eq!(Feature::parse(wire), Some(feature));
    }
}
