//! Scheduled posts over the REST transport.

use crate::actions::ScheduleActions;
use crate::model::{Draft, ScheduledStatus};
use crate::rest::{RestInstance, Transport};
use crate::{Error, Result};

impl<T: Transport> ScheduleActions for RestInstance<T> {
    type Error = Error;

    async fn scheduled(&self) -> Result<Vec<ScheduledStatus>> {
        let url = self.url("/api/v1/scheduled_statuses")?;
        self.fetch(self.get(url)).await
    }

    async fn schedule(&self, draft: &Draft, at: &str) -> Result<ScheduledStatus> {
        let url = self.url("/api/v1/statuses")?;
        let mut form = super::status_form(draft, false);
        form.push(("scheduled_at", at.to_owned()));
        let request = super::attach_idempotency_key(self.post_form(url, &form), draft);
        self.fetch(request).await
    }

    async fn cancel_scheduled(&self, id: &str) -> Result<()> {
        let url = self.url(&format!("/api/v1/scheduled_statuses/{id}"))?;
        // The answer is an empty body; decoding it as an object would make a successful
        // cancellation look like a failure.
        self.send_empty(self.delete(url)).await
    }
}
