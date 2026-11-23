//! The protocol-neutral Nodus client core.
//!
//! This crate owns what a client needs regardless of who serves it:
//!
//! * [`model`] — the shared Actor/Post shapes and AP-aligned vocabulary;
//! * [`api::InstanceApi`] — the read surface the client state machine drives;
//! * [`sync_api::SyncApi`] — the read-only crawl surface;
//! * [`actions`] — the operations a signed-in person can perform.
//!
//! No HTTP stack, vendor endpoint, or Mastodon-only field belongs here. Mastodon
//! and Misskey implement these traits in their own compatibility crates, and

pub mod actions;
pub mod api;
pub mod model;
pub mod rest;
pub mod sync_api;

pub use api::{InstanceApi, Streaming};
pub use model::*;
pub use sync_api::SyncApi;

use url::Url;

/// Why a client operation failed.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The instance answered with a non-success status.
    #[error("instance returned {status}: {message}")]
    Status {
        /// HTTP status code.
        status: u16,
        /// Error text from the instance, if any.
        message: String,
    },
    /// The instance rate-limited us; callers must honour the delay.
    #[error("rate limited, retry after {retry_after_s}s")]
    RateLimited {
        /// Seconds to wait before the next request.
        retry_after_s: u32,
    },
    /// The token was rejected; sync must stop until the user re-authorizes.
    #[error("token rejected (revoked or insufficient scope)")]
    Unauthorized,
    /// A response body did not match the expected shape.
    #[error("malformed response: {0}")]
    Decode(String),
    /// The request could not be completed at the transport layer.
    #[error("transport error: {0}")]
    Transport(String),
    /// The instance API does not expose this feature.
    #[error("{0} is not supported by this instance API")]
    Unsupported(&'static str),
    /// The server refused a preferences write, naming the key and allowed values.
    #[error("preference update refused: {0}")]
    PreferenceRefused(PreferenceRefusal),
    /// The capability model predicted this server cannot serve `verb`, and the
    /// server confirmed it with 404 or 501.
    ///
    /// Distinct from [`Error::Unsupported`], which a transport raises when it knows
    /// its own implementation lacks an operation. This one is only produced when the
    /// negotiated answer agrees with the failure, so it is trustworthy as a reason to
    /// degrade; a 404 on a *supported* verb stays [`Error::Status`] because that is a
    /// bug, not a limit. See `nodus_protocol::unsupported::capability_error`.
    #[error("{verb} is not served: this server does not support {feature}")]
    UnsupportedCapability {
        /// Capability name, matching `Feature::as_str` in `nodus-protocol`.
        feature: &'static str,
        /// The request that failed, e.g. `GET /api/v1/favourites`.
        verb: &'static str,
    },
}

/// Result alias used by every operation in this crate.
pub type Result<T> = core::result::Result<T, Error>;

/// A registered OAuth application on one instance.
///
/// The web client registers its own application (dynamic registration) and
/// keeps the client id in browser storage; the sync worker reuses whatever was
/// registered for the instance it is talking to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRegistration {
    /// Public client id.
    pub client_id: String,
    /// Secret, only for confidential clients; PKCE clients ignore it.
    pub client_secret: Option<String>,
    /// Redirect URI this app was registered with.
    pub redirect_uri: String,
}

/// An instance this client can talk to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    /// Base URL, e.g. `https://example.social`.
    pub base: Url,
}

impl Instance {
    /// Parse a host or URL into an instance, defaulting to `https`.
    ///
    /// Only `https` is accepted for remote instances; `http` is accepted only
    /// for loopback hosts (`localhost`, `*.localhost`, `127.0.0.0/8`, `::1`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`] when the input is not a usable `https` (or
    /// loopback `http`) URL.
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        let candidate = if trimmed.contains("://") {
            trimmed.to_owned()
        } else {
            format!("https://{trimmed}")
        };
        let base = Url::parse(&candidate).map_err(|e| Error::Transport(e.to_string()))?;
        match base.scheme() {
            "https" if base.has_host() => Ok(Self { base }),
            "http" if is_loopback(&base) => Ok(Self { base }),
            "http" => Err(Error::Transport(
                "plain http is only allowed for loopback hosts".to_owned(),
            )),
            scheme => Err(Error::Transport(format!(
                "unsupported instance URL scheme: {scheme}"
            ))),
        }
    }
}

fn is_loopback(url: &Url) -> bool {
    match url.host() {
        Some(url::Host::Domain(domain)) => {
            domain.eq_ignore_ascii_case("localhost") || domain.ends_with(".localhost")
        }
        Some(url::Host::Ipv4(addr)) => addr.is_loopback(),
        Some(url::Host::Ipv6(addr)) => addr.is_loopback(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_parse_accepts_https_and_loopback_http() {
        assert_eq!(
            Instance::parse("mastodon.example").unwrap().base.as_str(),
            "https://mastodon.example/"
        );
        assert_eq!(
            Instance::parse("https://mastodon.example/sub")
                .unwrap()
                .base
                .as_str(),
            "https://mastodon.example/sub"
        );
        assert_eq!(
            Instance::parse("http://localhost:3000")
                .unwrap()
                .base
                .as_str(),
            "http://localhost:3000/"
        );
        assert_eq!(
            Instance::parse("http://127.0.0.1:8080")
                .unwrap()
                .base
                .as_str(),
            "http://127.0.0.1:8080/"
        );
        assert_eq!(
            Instance::parse("http://[::1]:8080").unwrap().base.as_str(),
            "http://[::1]:8080/"
        );
        assert_eq!(
            Instance::parse("http://dev.localhost:3000")
                .unwrap()
                .base
                .as_str(),
            "http://dev.localhost:3000/"
        );
    }

    #[test]
    fn instance_parse_rejects_unsafe_schemes_and_remote_http() {
        for bad in [
            "http://mastodon.example",
            "http://192.168.1.10:3000",
            "http://evillocalhost",
            "javascript://alert(1)",
            "javascript:alert(1)",
            "file:///etc/passwd",
            "ftp://mastodon.example",
            "ws://localhost:3000",
            "wss://mastodon.example",
            "https://",
            "",
        ] {
            assert!(
                Instance::parse(bad).is_err(),
                "expected Instance::parse({bad:?}) to fail"
            );
        }
    }
}
