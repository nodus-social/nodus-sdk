//! Tests for the capability vocabulary itself.

use super::*;

#[test]
fn every_feature_spelling_round_trips() {
    for feature in Feature::ALL {
        assert_eq!(Feature::parse(feature.as_str()), Some(feature));
    }
}

#[test]
fn no_two_features_share_a_wire_spelling() {
    let mut names: Vec<&str> = Feature::ALL
        .iter()
        .map(|feature| feature.as_str())
        .collect();
    names.sort_unstable();
    let mut unique = names.clone();
    unique.dedup();
    assert_eq!(
        names, unique,
        "a duplicate wire spelling reuses an existing bit"
    );
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
fn every_name_in_the_vocabulary_parses() {
    // The vocabulary this build carries. `featured_tags` is published by
    // `nodus-backend/crates/api/src/meta.rs`; `notification_settings`, `sessions`,
    // `two_factor` and `exports` are named ahead of a backend route group, which is
    // why the baseline records them without a Nodus answer.
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
        "admin_overview",
        "admin_announcements",
        "admin_audit",
        "admin_domains",
        "admin_email",
        "admin_escalation",
        "admin_federation",
        "admin_protocol",
        "admin_replies",
        "admin_rules",
        "admin_scanners",
        "admin_settings",
        "admin_topology",
        "moderation_accounts",
        "moderation_appeals",
        "moderation_federation",
        "moderation_flags",
        "moderation_media",
        "moderation_registrations",
        "moderation_reports",
        "mutes",
        "blocks",
        "domain_blocks",
        "sessions",
        "two_factor",
        "exports",
        "featured_tags",
        "notification_settings",
        "pages",
    ];
    assert_eq!(published.len(), Feature::COUNT);
    for name in published {
        assert!(Feature::parse(name).is_some(), "{name} did not parse");
    }
}

#[test]
fn the_vocabulary_is_exactly_the_named_list() {
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
        "admin_overview",
        "admin_announcements",
        "admin_audit",
        "admin_domains",
        "admin_email",
        "admin_escalation",
        "admin_federation",
        "admin_protocol",
        "admin_replies",
        "admin_rules",
        "admin_scanners",
        "admin_settings",
        "admin_topology",
        "moderation_accounts",
        "moderation_appeals",
        "moderation_federation",
        "moderation_flags",
        "moderation_media",
        "moderation_registrations",
        "moderation_reports",
        "mutes",
        "blocks",
        "domain_blocks",
        "sessions",
        "two_factor",
        "exports",
        "featured_tags",
        "notification_settings",
        "pages",
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
