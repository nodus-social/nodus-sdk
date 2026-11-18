//! The v1 filter surface, for instances that predate v2's keyword lists.

use crate::actions::FilterV1Actions;
use crate::model::{FilterV1, FilterV1Draft};
use crate::rest::{RestInstance, Transport};
use crate::{Error, Result};

/// The fields a v1 filter write sends.
///
/// `context[]` is repeated rather than joined: the instance reads a single string as one
/// context, so joining them would silently narrow every filter to whichever came first.
fn filter_v1_form(draft: &FilterV1Draft) -> Vec<(&'static str, String)> {
    let mut form = vec![("phrase", draft.phrase.clone())];
    for context in &draft.context {
        form.push(("context[]", context.as_str().to_owned()));
    }
    if let Some(expires_in) = &draft.expires_in {
        form.push(("expires_in", expires_in.clone()));
    }
    form.push(("irreversible", draft.irreversible.to_string()));
    form.push(("whole_word", draft.whole_word.to_string()));
    form
}

impl<T: Transport> FilterV1Actions for RestInstance<T> {
    type Error = Error;

    async fn filters_v1(&self) -> Result<Vec<FilterV1>> {
        let url = self.url("/api/v1/filters")?;
        self.fetch(self.get(url)).await
    }

    async fn create_filter_v1(&self, draft: &FilterV1Draft) -> Result<FilterV1> {
        let url = self.url("/api/v1/filters")?;
        self.fetch(self.post_form(url, &filter_v1_form(draft)))
            .await
    }

    async fn update_filter_v1(&self, id: &str, draft: &FilterV1Draft) -> Result<FilterV1> {
        let url = self.url_segments(&["api", "v1", "filters", id])?;
        self.fetch(self.put_form(url, &filter_v1_form(draft))).await
    }

    async fn delete_filter_v1(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "filters", id])?;
        self.send_empty(self.delete(url)).await
    }
}
