//! What the instance recommends: accounts to follow and links being shared.
//!
//! Suggestions and trending links are the two reads a "find something new" screen is
//! built from. Both are the instance's ranking, not this client's, so the shapes here
//! stay close to the wire and carry the source with them — a reader is entitled to know
//! whether an account was suggested by staff or by the algorithm.

use serde::{Deserialize, Serialize};

use super::{Account, TagHistory};

/// An account the instance suggests following.
// Partial equality only, because the account carries a picture shape whose aspect is a
// float.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Suggestion {
    pub account: Account,
    /// Why it was suggested (`staff`, `global`, `friends_of_friends`, ...).
    #[serde(default)]
    pub source: String,
    /// Every reason it was suggested, on instances that report more than one.
    #[serde(default)]
    pub sources: Vec<String>,
}

/// A link many people on the instance are sharing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendingLink {
    pub url: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// The card kind: `link`, `photo`, `video` or `rich`.
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub author_url: String,
    #[serde(default)]
    pub provider_name: String,
    #[serde(default)]
    pub provider_url: String,
    /// The provider's embeddable HTML, when it offers one.
    #[serde(default)]
    pub html: String,
    #[serde(default)]
    pub width: u64,
    #[serde(default)]
    pub height: u64,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub embed_url: String,
    /// Blurhash of `image`, for showing a placeholder before it loads.
    #[serde(default)]
    pub blurhash: Option<String>,
    /// Daily usage counts, on the trends endpoint only.
    #[serde(default)]
    pub history: Vec<TagHistory>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_suggestion_parses_its_account_and_source() {
        let suggestion: Suggestion = serde_json::from_str(
            r#"{"account":{"id":"1","acct":"alice","display_name":"Alice"},"source":"staff"}"#,
        )
        .expect("suggestion payload");
        assert_eq!(suggestion.account.acct, "alice");
        assert_eq!(suggestion.source, "staff");
        assert_eq!(suggestion.sources.len(), 0);
    }

    #[test]
    fn a_trending_link_parses_the_card_type_renamed_field() {
        let link: TrendingLink = serde_json::from_str(
            r#"{"url":"https://example.com/a","title":"A","type":"link","history":[]}"#,
        )
        .expect("trending link payload");
        assert_eq!(link.kind, "link");
        assert_eq!(link.title, "A");
    }
}
