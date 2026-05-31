use crate::client::Client;
use crate::types::TagDatabasePage;
use crate::{Error, Mac, TagRecord};

impl Client {
    /// Retrieve all tags from the AP, following pagination automatically.
    pub async fn get_tags(&self) -> Result<Vec<TagRecord>, Error> {
        let mut all_tags = Vec::new();
        let mut pos = 0u32;

        loop {
            let page = self.get_tags_page(pos).await?;
            all_tags.extend(page.tags);

            match page.continuation {
                Some(next) if next > pos => pos = next,
                _ => return Ok(all_tags),
            }
        }
    }

    /// Retrieve a single tag by MAC address.
    pub async fn get_tag(&self, mac: &Mac) -> Result<TagRecord, Error> {
        let url = self.url_with_params("/get_db", &[("mac", &mac.to_string())]);
        let page: TagDatabasePage = self.http.get(&url).send().await?.json().await?;

        page.tags.into_iter().next().ok_or(Error::Api {
            message: format!("tag {mac} not found"),
        })
    }

    pub(crate) async fn get_tags_page(&self, pos: u32) -> Result<TagDatabasePage, Error> {
        let pos_str = pos.to_string();
        let url = self.url_with_params("/get_db", &[("pos", &pos_str)]);
        let page: TagDatabasePage = self.http.get(&url).send().await?.json().await?;
        Ok(page)
    }
}
