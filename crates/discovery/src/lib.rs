//! Client for the Nodus discovery API: our recommendation service, not Mastodon.
//!
//! The two planes are deliberately separate. `nodus-instance` (the `native`/`wasm` transports) talks to the
//! user's server; this crate talks to us, over JSON REST with a Mastodon bearer
//! token and a `X-Contract-Version` header on every request. Keeping the transport
//! out of `nodus-sdk` is what lets the state machine compile without `reqwest` or
//! `gloo-net`, and it is why this is a sibling crate rather than a module behind a
//! feature of the SDK. The `Discovery` trait itself lives in `nodus-state`, so this
//! crate can implement it without depending on the facade.
//!
//! Enable `native` for `DiscoveryClient` (`reqwest`, the calculator and native
//! tests) or `wasm` for `BrowserDiscovery` (`gloo-net` and `web-sys`, the web
//! client). Both map statuses the same way. In particular, a `426` becomes
//! [`Error::ClientTooOld`] instead of a generic [`Error::Status`], because an old
//! contract needs an "update the app" screen and not a retry loop.

/// The error a discovery request can fail with.
///
/// The variants are deliberately coarse: the SDK and the UI branch on *what to
/// do* (re-authorise, back off, update, retry), not on the API's `code` string.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The API answered with a non-success status.
    #[error("discovery returned {status}: {message}")]
    Status {
        /// HTTP status code, for logging and for the rare status-specific screen.
        status: u16,
        /// The API's message when it sent the documented error envelope.
        message: String,
    },
    /// The API rate-limited us; the caller must honour the delay.
    #[error("rate limited, retry after {retry_after_s}s")]
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after_s: u64,
    },
    /// The token was rejected, so the feed cannot make progress.
    ///
    /// This is separate from [`Status`](Self::Status) because the UI has to send
    /// the user back through the instance's authorisation flow.
    #[error("token rejected (revoked or insufficient scope)")]
    Unauthorized,
    /// The server no longer accepts this build's contract version.
    ///
    /// The UI must render an "update required" screen; retrying cannot help.
    #[error("client is too old: contract {server} requires {min}")]
    ClientTooOld {
        /// The contract version involved in the rejection: the server's version
        /// when the response reports it, otherwise the version this client sent.
        server: String,
        /// The oldest contract version the server accepts.
        min: String,
    },
    /// A response body did not match the shape this client expects.
    ///
    /// The serde message is kept, the raw body is not: a body can echo the
    /// request, including a URL that carries a cursor.
    #[error("malformed response: {0}")]
    Decode(String),
    /// The request could not be completed at the transport layer.
    #[error("transport error: {0}")]
    Transport(String),
}

/// Result alias with the discovery [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(feature = "native")]
pub mod client;
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod wasm;

#[cfg(feature = "native")]
pub use client::DiscoveryClient;
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub use wasm::BrowserDiscovery;

/// Resolves a static discovery path against `base` using `Url`'s path-segment writer.
///
/// Preserves any path prefix on `base` and rejects empty or dot segments so path
/// characters cannot alter the origin or route shape.
#[cfg(any(
    feature = "native",
    all(target_arch = "wasm32", feature = "wasm"),
    test
))]
pub(crate) fn resolve_url(base: &url::Url, path: &str) -> Result<url::Url> {
    let mut url = base.clone();
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|()| Error::Transport("the API base cannot carry path segments".to_owned()))?;
        segments.pop_if_empty();
        let trimmed = path.trim_start_matches('/');
        if !trimmed.is_empty() {
            for segment in trimmed.split('/') {
                if matches!(segment, "" | "." | "..") {
                    return Err(Error::Transport(format!(
                        "{path}: invalid path segment {segment:?}"
                    )));
                }
                segments.push(segment);
            }
        }
    }
    Ok(url)
}

/// Builds `/api/v1/feed/{feed_id}/{action}` against `base`, percent-encoding
/// `feed_id` as a single path segment.
#[cfg(any(
    feature = "native",
    all(target_arch = "wasm32", feature = "wasm"),
    test
))]
pub(crate) fn feed_action_url(base: &url::Url, feed_id: &str, action: &str) -> Result<url::Url> {
    if matches!(feed_id, "" | "." | "..") || matches!(action, "" | "." | "..") {
        return Err(Error::Transport(
            "invalid feed action path segment".to_owned(),
        ));
    }
    let mut url = resolve_url(base, "/api/v1/feed")?;
    {
        let mut segments = url.path_segments_mut().map_err(|()| {
            Error::Transport("/api/v1/feed: the API base cannot carry path segments".to_owned())
        })?;
        segments.push(feed_id).push(action);
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::{feed_action_url, resolve_url};

    #[test]
    fn resolve_url_preserves_base_path_prefix() {
        let base = Url::parse("https://discovery.example/v2/prefix").expect("valid base");
        let resolved = resolve_url(&base, "/api/v1/meta").expect("resolved");
        assert_eq!(
            resolved.as_str(),
            "https://discovery.example/v2/prefix/api/v1/meta"
        );
    }

    #[test]
    fn feed_action_url_blocks_traversal_origin_override_and_dot_segments() {
        let base = Url::parse("https://discovery.example/v2").expect("valid base");

        let traversed =
            feed_action_url(&base, "../admin?x=1#frag", "refresh").expect("encoded segment");
        assert_eq!(traversed.host_str(), Some("discovery.example"));
        assert_eq!(
            traversed.path(),
            "/v2/api/v1/feed/..%2Fadmin%3Fx=1%23frag/refresh"
        );
        assert_eq!(traversed.query(), None);
        assert_eq!(traversed.fragment(), None);

        let origin_override =
            feed_action_url(&base, "//evil.example/steal", "refresh").expect("encoded segment");
        assert_eq!(origin_override.host_str(), Some("discovery.example"));
        assert_eq!(
            origin_override.path(),
            "/v2/api/v1/feed/%2F%2Fevil.example%2Fsteal/refresh"
        );

        for bad in ["", ".", ".."] {
            assert!(
                feed_action_url(&base, bad, "refresh").is_err(),
                "expected {bad:?} to be rejected"
            );
        }
    }
}
