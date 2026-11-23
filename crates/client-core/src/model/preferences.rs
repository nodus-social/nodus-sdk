//! The account's posting and reading defaults, typed.
//!
//! Mastodon spells the wire keys with colons (`posting:default:visibility`), so the
//! field renames live here and a caller never handles the strings. The three choices
//! are closed enums over the values the server accepts; an unfamiliar value read from
//! a newer server falls back to the server default with a warning, because failing the
//! whole read would make the settings screen unusable over one unknown key.

use core::fmt;
use core::str::FromStr;

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

use super::Visibility;

mod patch;
mod refusal;

pub use patch::PreferencesPatch;
pub use refusal::PreferenceRefusal;

/// Who may quote a new post.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum QuotePolicy {
    /// Anyone who can see the post.
    #[default]
    Public,
    /// Followers of the author.
    Followers,
    /// Nobody.
    Nobody,
}

impl fmt::Display for QuotePolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Public => "public",
            Self::Followers => "followers",
            Self::Nobody => "nobody",
        })
    }
}

impl FromStr for QuotePolicy {
    type Err = UnknownPreferenceValue;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "public" => Ok(Self::Public),
            "followers" => Ok(Self::Followers),
            "nobody" => Ok(Self::Nobody),
            other => Err(UnknownPreferenceValue(other.to_owned())),
        }
    }
}

impl<'de> Deserialize<'de> for QuotePolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(value.parse().unwrap_or_else(|_| {
            warn_unknown("posting:default:quote_policy", &value);
            Self::default()
        }))
    }
}

/// How media is expanded in timelines.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpandMedia {
    /// The instance's default.
    #[default]
    Default,
    /// Expand every attachment.
    ShowAll,
    /// Keep every attachment collapsed.
    HideAll,
}

impl fmt::Display for ExpandMedia {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Default => "default",
            Self::ShowAll => "show_all",
            Self::HideAll => "hide_all",
        })
    }
}

impl FromStr for ExpandMedia {
    type Err = UnknownPreferenceValue;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "default" => Ok(Self::Default),
            "show_all" => Ok(Self::ShowAll),
            "hide_all" => Ok(Self::HideAll),
            other => Err(UnknownPreferenceValue(other.to_owned())),
        }
    }
}

impl<'de> Deserialize<'de> for ExpandMedia {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(value.parse().unwrap_or_else(|_| {
            warn_unknown("reading:expand:media", &value);
            Self::default()
        }))
    }
}

/// A wire value that is not one of the spellings this client knows.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown preference value: {0}")]
pub struct UnknownPreferenceValue(pub String);

/// The account's own defaults, as `GET /api/v1/preferences` reports them.
#[allow(
    clippy::struct_excessive_bools,
    reason = "the wire schema has four independent booleans"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    /// What the composer should start with.
    #[serde(
        rename = "posting:default:visibility",
        deserialize_with = "deserialize_visibility"
    )]
    pub posting_default_visibility: Visibility,
    /// Whether new posts start sensitive.
    #[serde(rename = "posting:default:sensitive")]
    pub posting_default_sensitive: bool,
    /// The language the composer should preselect.
    #[serde(rename = "posting:default:language")]
    pub posting_default_language: Option<String>,
    /// Who may quote a new post.
    #[serde(rename = "posting:default:quote_policy")]
    pub posting_default_quote_policy: QuotePolicy,
    /// Whether media is expanded rather than collapsed.
    #[serde(rename = "reading:expand:media")]
    pub reading_expand_media: ExpandMedia,
    /// Whether content warnings start open.
    #[serde(rename = "reading:expand:spoilers")]
    pub reading_expand_spoilers: bool,
    /// Whether GIFs autoplay.
    #[serde(rename = "reading:autoplay:gifs")]
    pub reading_autoplay_gifs: bool,
    /// Whether the account receives notification email.
    #[serde(rename = "notification:email")]
    pub notification_email: bool,
    /// Whether the account receives report notifications.
    #[serde(rename = "notification:report")]
    pub notification_report: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            posting_default_visibility: Visibility::Public,
            posting_default_sensitive: false,
            posting_default_language: None,
            posting_default_quote_policy: QuotePolicy::Public,
            reading_expand_media: ExpandMedia::Default,
            reading_expand_spoilers: false,
            reading_autoplay_gifs: false,
            notification_email: true,
            notification_report: true,
        }
    }
}

impl Preferences {
    /// The visibility a new post should start with.
    #[must_use]
    pub const fn visibility(&self) -> Visibility {
        self.posting_default_visibility
    }
}

/// Reads the visibility field without making one unknown value fail the whole read.
fn deserialize_visibility<'de, D>(deserializer: D) -> Result<Visibility, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(match value.as_deref() {
        Some("public") | None => Visibility::Public,
        Some("unlisted") => Visibility::Unlisted,
        Some("private") => Visibility::Private,
        Some("direct") => Visibility::Direct,
        Some(other) => {
            warn_unknown("posting:default:visibility", other);
            Visibility::Public
        }
    })
}

/// Says why a value was ignored; there is no logging dependency in this crate.
fn warn_unknown(key: &str, value: &str) {
    eprintln!("nodus: unknown value {value:?} for preference {key}; using the default");
}
