//! What the server says it can do.
//!
//! Nodus answers from `GET /api/v1/meta`, which we own: a Mastodon instance would 404
//! it, and that 404 is itself the answer ("no extensions"). Mastodon and Misskey
//! publish no capability list at all, so for them the SDK answers from a documented
//! baseline in [`crate::baseline`] instead of pretending "no answer" means "nothing".
//!
//! Parsing is tolerant in both directions. A name the server advertises that this
//! build does not know is kept in [`Capabilities::unknown`] rather than dropped, so a
//! newer server still informs an older client; a name this build knows but the server
//! omits simply reads as unsupported. The active answer itself is an
//! [`ActiveFeatures`] set, not strings matched at request time.

use serde::{Deserialize, Serialize};

pub use crate::active::ActiveFeatures;
pub use crate::feature::Feature;

/// Which protocol a server speaks, as far as capability answers go.
///
/// The distinction decides where the answer comes from: Nodus publishes a list, the
/// other two are answered from the SDK's [`crate::baseline`] because they publish nothing.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    /// The Nodus extensions, where the published `features` list is authoritative.
    Nodus,
    /// The Mastodon client API (`/api/v1/*`).
    #[default]
    Mastodon,
    /// The Misskey client API (`/api/*`).
    Misskey,
}

impl Provider {
    /// The name used in logs, settings and state.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nodus => "nodus",
            Self::Mastodon => "mastodon",
            Self::Misskey => "misskey",
        }
    }
}

/// The negotiated capabilities of one server.
///
/// `contract` is the version of *our* wire contract (`0.1.0` today); a client that
/// needs a newer one must say so instead of guessing from the feature list, because a
/// feature name can be reused across a breaking change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// Contract version the server speaks, e.g. `0.1.0`.
    pub contract: String,
    /// The provider whose answer this is.
    #[serde(default)]
    pub provider: Provider,
    /// The typed active set. For Nodus this is built from the published names; for the
    /// other two it is the judged baseline, so no caller has to re-derive it.
    #[serde(default)]
    pub features: ActiveFeatures,
    /// Feature names the server advertised that this build does not know.
    ///
    /// Kept rather than dropped: a newer server may gate a surface behind a name an
    /// older client has never heard of, and the caller can still show that name in a
    /// diagnostics screen instead of silently losing it.
    #[serde(default)]
    pub unknown: Vec<String>,
}

impl Default for Capabilities {
    /// The compatibility floor: Mastodon, with the baseline already active.
    fn default() -> Self {
        Self::mastodon()
    }
}

impl Capabilities {
    /// A plain Mastodon instance: no Nodus extensions, but its baseline surfaces work.
    ///
    /// Also the value before anything has been negotiated, which is why it is the
    /// default: the compatibility floor is the only answer that is never a lie.
    #[must_use]
    pub fn mastodon() -> Self {
        Self {
            contract: String::new(),
            provider: Provider::Mastodon,
            features: crate::baseline::active(Provider::Mastodon),
            unknown: Vec::new(),
        }
    }

    /// A Misskey instance, answered from the documented baseline.
    #[must_use]
    pub fn misskey() -> Self {
        Self {
            contract: String::new(),
            provider: Provider::Misskey,
            features: crate::baseline::active(Provider::Misskey),
            unknown: Vec::new(),
        }
    }

    /// The provider whose baseline or list answers [`Capabilities::supports`].
    #[must_use]
    pub const fn provider(&self) -> Provider {
        self.provider
    }

    /// Whether the server runs Nodus's own extensions.
    ///
    /// An empty contract is the Mastodon surface: either the server answered nothing, or
    /// it answered with a protocol version this client does not recognise. Callers use
    /// this to skip a request that can only 404 there.
    #[must_use]
    pub fn knows_extensions(&self) -> bool {
        self.provider == Provider::Nodus && !self.contract.is_empty()
    }

    /// Whether the server serves `feature`.
    ///
    /// A bit test over the typed active set: Nodus's set came from its published list,
    /// Mastodon's and Misskey's from [`crate::baseline`]. There is no second runtime
    /// lookup to disagree with the negotiation answer.
    #[must_use]
    pub fn supports(&self, feature: Feature) -> bool {
        self.features.contains(feature)
    }

    /// Whether every feature in `needed` is served.
    #[must_use]
    pub fn supports_all(&self, needed: &[Feature]) -> bool {
        needed.iter().all(|feature| self.supports(*feature))
    }

    /// Builds a Nodus answer from the names `/api/v1/meta` published.
    ///
    /// The provider is Nodus by definition. A name this build cannot spell is retained
    /// in [`Capabilities::unknown`], never dropped: a newer server must still be able
    /// to tell an older client what it offers. The contract is empty here; use
    /// [`Capabilities::from_parts`] when the document carried one.
    #[must_use]
    pub fn from_published(names: &[String]) -> Self {
        let mut features = ActiveFeatures::NONE;
        let mut unknown = Vec::new();
        for name in names {
            match Feature::parse(name) {
                Some(feature) => features.insert(feature),
                None => unknown.push(name.clone()),
            }
        }
        Self {
            contract: String::new(),
            provider: Provider::Nodus,
            features,
            unknown,
        }
    }

    /// Reads a `/api/v1/meta` answer the SDK already decoded into a typed `Meta`.
    ///
    /// The provider is Nodus by definition: only our API publishes this document.
    #[must_use]
    pub fn from_parts(contract: impl Into<String>, names: &[String]) -> Self {
        Self {
            contract: contract.into(),
            ..Self::from_published(names)
        }
    }

    /// Reads a `/api/v1/meta` document.
    ///
    /// # Errors
    ///
    /// Returns an error only when the document is not an object — a missing
    /// `features` list means "no extensions", which is a valid answer, not a failure.
    pub fn from_meta(value: &serde_json::Value) -> Result<Self, MetaError> {
        let object = value.as_object().ok_or(MetaError::NotAnObject)?;
        let contract = object
            .get("contract")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let names: Vec<String> = object
            .get("features")
            .and_then(serde_json::Value::as_array)
            .map(|names| {
                names
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self::from_parts(contract, &names))
    }
}

/// Why a meta document could not be read.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MetaError {
    /// The body was JSON but not an object (an array, `null`, a bare string).
    #[error("a meta document must be a JSON object")]
    NotAnObject,
}

#[cfg(test)]
mod tests;
