//! A partial write: only the keys the caller changed.

use serde::Serialize;

use super::{ExpandMedia, Preferences, QuotePolicy};
use crate::model::Visibility;

/// A partial update: only keys set here are written.
///
/// Build it with [`between`](Self::between) so the body carries only the values the
/// caller actually changed. That is what keeps a read-modify-write from clobbering a
/// key another client changed after the read; a full-object `PUT` would not.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct PreferencesPatch {
    /// New default visibility.
    #[serde(
        rename = "posting:default:visibility",
        skip_serializing_if = "Option::is_none"
    )]
    pub posting_default_visibility: Option<Visibility>,
    /// New sensitive default.
    #[serde(
        rename = "posting:default:sensitive",
        skip_serializing_if = "Option::is_none"
    )]
    pub posting_default_sensitive: Option<bool>,
    /// New default language; `Some(None)` clears it with an explicit JSON `null`.
    #[serde(
        rename = "posting:default:language",
        skip_serializing_if = "Option::is_none"
    )]
    pub posting_default_language: Option<Option<String>>,
    /// New quote policy.
    #[serde(
        rename = "posting:default:quote_policy",
        skip_serializing_if = "Option::is_none"
    )]
    pub posting_default_quote_policy: Option<QuotePolicy>,
    /// New media expansion default.
    #[serde(
        rename = "reading:expand:media",
        skip_serializing_if = "Option::is_none"
    )]
    pub reading_expand_media: Option<ExpandMedia>,
    /// New spoiler default.
    #[serde(
        rename = "reading:expand:spoilers",
        skip_serializing_if = "Option::is_none"
    )]
    pub reading_expand_spoilers: Option<bool>,
    /// New GIF autoplay default.
    #[serde(
        rename = "reading:autoplay:gifs",
        skip_serializing_if = "Option::is_none"
    )]
    pub reading_autoplay_gifs: Option<bool>,
    /// New email-notification default.
    #[serde(rename = "notification:email", skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<bool>,
    /// New report-notification default.
    #[serde(
        rename = "notification:report",
        skip_serializing_if = "Option::is_none"
    )]
    pub notification_report: Option<bool>,
}

impl PreferencesPatch {
    /// The keys and values that differ between `current` and `desired`.
    #[must_use]
    pub fn between(current: &Preferences, desired: &Preferences) -> Self {
        Self {
            posting_default_visibility: (current.posting_default_visibility
                != desired.posting_default_visibility)
                .then_some(desired.posting_default_visibility),
            posting_default_sensitive: (current.posting_default_sensitive
                != desired.posting_default_sensitive)
                .then_some(desired.posting_default_sensitive),
            posting_default_language: (current.posting_default_language
                != desired.posting_default_language)
                .then_some(desired.posting_default_language.clone()),
            posting_default_quote_policy: (current.posting_default_quote_policy
                != desired.posting_default_quote_policy)
                .then_some(desired.posting_default_quote_policy),
            reading_expand_media: (current.reading_expand_media != desired.reading_expand_media)
                .then_some(desired.reading_expand_media),
            reading_expand_spoilers: (current.reading_expand_spoilers
                != desired.reading_expand_spoilers)
                .then_some(desired.reading_expand_spoilers),
            reading_autoplay_gifs: (current.reading_autoplay_gifs != desired.reading_autoplay_gifs)
                .then_some(desired.reading_autoplay_gifs),
            notification_email: (current.notification_email != desired.notification_email)
                .then_some(desired.notification_email),
            notification_report: (current.notification_report != desired.notification_report)
                .then_some(desired.notification_report),
        }
    }

    /// Whether this patch would send no keys at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.posting_default_visibility.is_none()
            && self.posting_default_sensitive.is_none()
            && self.posting_default_language.is_none()
            && self.posting_default_quote_policy.is_none()
            && self.reading_expand_media.is_none()
            && self.reading_expand_spoilers.is_none()
            && self.reading_autoplay_gifs.is_none()
            && self.notification_email.is_none()
            && self.notification_report.is_none()
    }
}
