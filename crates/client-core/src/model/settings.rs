//! Instance announcements.
//!
//! Announcements are what the *instance* wants said, which is why they are dismissible
//! per account. Account preferences live in their own module.

use serde::{Deserialize, Serialize};

/// One message from the instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Announcement {
    pub id: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub published: bool,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub starts_at: Option<String>,
    #[serde(default)]
    pub ends_at: Option<String>,
    #[serde(default)]
    pub read: Option<bool>,
}

impl Announcement {
    /// Whether this one is worth showing: published, unread, and not finished.
    ///
    /// `ends_at` is compared as a string, which is exact here because the API sends
    /// UTC timestamps in one format; parsing a date would need a calendar for no gain.
    pub fn should_show(&self, now: &str) -> bool {
        if !self.published || self.read == Some(true) {
            return false;
        }
        self.ends_at.as_ref().is_none_or(|ends| ends.as_str() > now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_read_announcement_is_not_shown_again() {
        let announcement = Announcement {
            id: "1".into(),
            content: "hello".into(),
            published: true,
            all_day: false,
            published_at: None,
            starts_at: None,
            ends_at: None,
            read: Some(true),
        };
        assert!(!announcement.should_show("2026-11-07T00:00:00Z"));
    }

    #[test]
    fn an_expired_announcement_is_not_shown() {
        let announcement = Announcement {
            id: "2".into(),
            content: "old news".into(),
            published: true,
            all_day: false,
            published_at: None,
            starts_at: None,
            ends_at: Some("2026-01-01T00:00:00Z".into()),
            read: None,
        };
        assert!(!announcement.should_show("2026-11-07T00:00:00Z"));
        assert!(announcement.should_show("2025-12-31T00:00:00Z"));
    }
}
