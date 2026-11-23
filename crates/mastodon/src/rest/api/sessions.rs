//! Signed-in sessions over the REST transport.
//!
//! The list and its revoke verbs are a Nodus surface (`/api/v1/auth/sessions`), served
//! beside the Mastodon routes because they act on the account's browser sessions rather
//! than on posts. The transport is shared with every other action, so a caller that can
//! make one request can make these without a second client.

use crate::actions::SessionActions;
use crate::model::AuthSession;
use crate::rest::{RestInstance, Transport};
use crate::{Error, Result};

/// Revokes every session in `scope` (`others` or `all`).
///
/// Kept as one helper because the two verbs differ only in the query value: spelling the
/// URL twice is how a client ends up revoking "all" when the reader asked for "others".
async fn revoke_scope<T: Transport>(instance: &RestInstance<T>, scope: &str) -> Result<()> {
    let mut url = instance.url("/api/v1/auth/sessions")?;
    url.query_pairs_mut().append_pair("scope", scope);
    instance.send_empty(instance.delete(url)).await
}

impl<T: Transport> SessionActions for RestInstance<T> {
    type Error = Error;

    async fn sessions(&self) -> Result<Vec<AuthSession>> {
        let url = self.url("/api/v1/auth/sessions")?;
        self.fetch(self.get(url)).await
    }

    async fn revoke_session(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "auth", "sessions", id])?;
        // A successful revoke answers 204 with no body, so decoding it as an object
        // would turn the one outcome the reader wanted into a failure.
        self.send_empty(self.delete(url)).await
    }

    async fn revoke_other_sessions(&self) -> Result<()> {
        revoke_scope(self, "others").await
    }

    async fn revoke_all_sessions(&self) -> Result<()> {
        revoke_scope(self, "all").await
    }
}
