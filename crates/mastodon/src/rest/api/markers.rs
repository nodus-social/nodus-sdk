//! Reading positions, over the REST transport.

use crate::actions::MarkerActions;
use crate::model::{MarkerTimeline, MarkerUpdate, Markers};
use crate::rest::{RestInstance, Transport};
use crate::{Error, Result};

impl<T: Transport> MarkerActions for RestInstance<T> {
    type Error = Error;

    async fn markers(&self, timelines: &[MarkerTimeline]) -> Result<Markers> {
        let mut url = self.url("/api/v1/markers")?;
        {
            let mut pairs = url.query_pairs_mut();
            for timeline in timelines {
                pairs.append_pair("timeline[]", timeline.as_str());
            }
        }
        self.fetch(self.get(url)).await
    }

    async fn save_markers(&self, updates: &[MarkerUpdate]) -> Result<Markers> {
        let url = self.url("/api/v1/markers")?;
        let form: Vec<(String, String)> = updates
            .iter()
            .flat_map(|update| {
                let name = update.timeline.as_str().to_owned();
                let mut fields =
                    vec![(format!("{name}[last_read_id]"), update.last_read_id.clone())];
                if let Some(version) = update.version {
                    fields.push((format!("{name}[version]"), version.to_string()));
                }
                fields
            })
            .collect();
        let borrowed: Vec<(&str, String)> = form
            .iter()
            .map(|(name, value)| (name.as_str(), value.clone()))
            .collect();
        self.fetch(self.post_form(url, &borrowed)).await
    }
}
