//! Preferences and announcements over the REST transport.

use crate::actions::SettingsActions;
use crate::model::{Announcement, PreferenceRefusal, Preferences, PreferencesPatch};
use crate::rest::{RestInstance, Transport, decode, error_message};
use crate::{Error, Result};

impl<T: Transport> SettingsActions for RestInstance<T> {
    type Error = Error;

    async fn preferences(&self) -> Result<Preferences> {
        let url = self.url("/api/v1/preferences")?;
        self.fetch(self.get(url)).await
    }

    async fn update_preferences(&self, patch: &PreferencesPatch) -> Result<Preferences> {
        let url = self.url("/api/v1/preferences")?;
        let body = serde_json::to_vec(patch).map_err(|error| Error::Decode(error.to_string()))?;
        let response = self.transport().send(self.put_body(url, body)).await?;
        if response.status == 422 {
            if let Some(refusal) = PreferenceRefusal::parse(&error_message(&response.body)) {
                return Err(Error::PreferenceRefused(refusal));
            }
        }
        decode(&response)
    }

    async fn announcements(&self) -> Result<Vec<Announcement>> {
        let url = self.url("/api/v1/announcements")?;
        self.fetch(self.get(url)).await
    }

    async fn dismiss_announcement(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "announcements", id, "dismiss"])?;
        // Dismissing answers 200 with an empty body; decoding it as an object would
        // make a successful dismissal look like a failure.
        self.send_empty(self.post(url)).await
    }
}
