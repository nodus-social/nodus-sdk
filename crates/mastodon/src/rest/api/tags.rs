//! Tags: the account's featured set, one tag's metadata, following a tag.

use crate::actions::TagActions;
use crate::model::{FeaturedTag, Tag};
use crate::rest::{RestInstance, Transport};
use crate::{Error, Result};

impl<T: Transport> TagActions for RestInstance<T> {
    type Error = Error;

    async fn own_featured_tags(&self) -> Result<Vec<FeaturedTag>> {
        let url = self.url("/api/v1/featured_tags")?;
        self.fetch(self.get(url)).await
    }

    async fn tag(&self, name: &str) -> Result<Tag> {
        let url = self.url_segments(&["api", "v1", "tags", name])?;
        self.fetch(self.get(url)).await
    }

    async fn follow_tag(&self, name: &str) -> Result<Tag> {
        let url = self.url_segments(&["api", "v1", "tags", name, "follow"])?;
        self.fetch(self.post(url)).await
    }

    async fn unfollow_tag(&self, name: &str) -> Result<Tag> {
        let url = self.url_segments(&["api", "v1", "tags", name, "unfollow"])?;
        self.fetch(self.post(url)).await
    }
}
