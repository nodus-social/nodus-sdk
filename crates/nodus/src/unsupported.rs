//! Turning "no such route" into an honest capability answer.
//!
//! A 404 alone is ambiguous: it can mean the server does not serve the surface, or it
//! can mean this client sent a broken request. The capability model resolves the
//! ambiguity, and only in one direction:
//!
//! * 404 or 501 for a verb the model says is **unsupported** → [`Error::UnsupportedCapability`];
//!   the answer matches the prediction, so the caller can degrade silently.
//! * 404 for a verb the model says is **supported** → `None`, and the transport keeps
//!   its normal [`Error::Status`]; that is a real bug, and hiding it as "unsupported"
//!   would make a broken instance look like a limited one.
//!
//! Any other status also returns `None`: 401, 403, 429 and 5xx carry their own
//! meanings and must not be laundered into "this server does not serve that".

use nodus_client_core::Error;

use crate::capabilities::Capabilities;
use crate::feature::Feature;

/// Maps a failed response to [`Error::UnsupportedCapability`] only when the
/// negotiation answer predicted it.
///
/// Returns `None` when the caller should keep its own status handling; see the module
/// documentation for the rule and why the "supported 404" case must stay a bug.
#[must_use]
pub fn capability_error(
    capabilities: &Capabilities,
    feature: Feature,
    verb: &'static str,
    status: u16,
) -> Option<Error> {
    if matches!(status, 404 | 501) && !capabilities.supports(feature) {
        Some(Error::UnsupportedCapability {
            feature: feature.as_str(),
            verb,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::Provider;

    const VERB: &str = "GET /api/v1/favourites";

    #[test]
    fn a_404_on_an_unsupported_verb_is_an_unsupported_capability() {
        let mastodon = Capabilities::mastodon();
        let error = capability_error(&mastodon, Feature::Batch, VERB, 404)
            .expect("a predicted miss is an answer");
        assert!(matches!(
            error,
            Error::UnsupportedCapability {
                feature: "batch",
                verb: VERB,
            }
        ));
    }

    #[test]
    fn a_501_on_an_unsupported_verb_is_also_a_capability_answer() {
        let misskey = Capabilities::misskey();
        assert!(capability_error(&misskey, Feature::Batch, VERB, 501).is_some());
    }

    #[test]
    fn a_404_on_a_supported_verb_stays_a_bug() {
        let mastodon = Capabilities::mastodon();
        assert!(capability_error(&mastodon, Feature::Polls, VERB, 404).is_none());
    }

    #[test]
    fn an_advertised_nodus_feature_that_404s_stays_a_bug() {
        let nodus = Capabilities::from_parts("0.1.0", &["batch".to_owned()]);
        assert_eq!(nodus.provider(), Provider::Nodus);
        assert!(capability_error(&nodus, Feature::Batch, VERB, 404).is_none());
        assert!(capability_error(&nodus, Feature::SyncToken, VERB, 404).is_some());
    }

    #[test]
    fn other_statuses_are_never_laundered_into_unsupported() {
        let mastodon = Capabilities::mastodon();
        for status in [400, 401, 403, 410, 429, 500, 502] {
            assert!(
                capability_error(&mastodon, Feature::Batch, VERB, status).is_none(),
                "{status} must reach the caller unchanged"
            );
        }
    }
}
