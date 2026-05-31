//! Tag management: listing, querying, commands, config, and database
//! backup/restore.

use crate::client::Client;
use crate::types::TagDatabasePage;
use crate::{Error, Mac, SaveTagConfig, TagCommand, TagRecord};

impl Client {
    /// Retrieve all tags from the AP, following pagination automatically.
    pub async fn get_tags(&self) -> Result<Vec<TagRecord>, Error> {
        const MAX_PAGES: u32 = 1024;

        let mut all_tags = Vec::new();
        let mut pos = 0u32;

        for _ in 0..MAX_PAGES {
            let page = self.get_tags_page(pos).await?;
            all_tags.extend(page.tags);

            match page.continuation {
                Some(next) if next > pos => pos = next,
                _ => return Ok(all_tags),
            }
        }

        Err(Error::Api {
            message: format!("pagination exceeded {MAX_PAGES} pages"),
        })
    }

    /// Retrieve a single tag by MAC address.
    pub async fn get_tag(&self, mac: &Mac) -> Result<TagRecord, Error> {
        let url = self.url_with_params("/get_db", &[("mac", &mac.to_string())]);
        let page: TagDatabasePage = self.http.get(&url).send().await?.error_for_status()?.json().await?;

        page.tags.into_iter().next().ok_or(Error::Api {
            message: format!("tag {mac} not found"),
        })
    }

    /// Send a command to a tag.
    pub async fn tag_cmd(&self, mac: &Mac, cmd: TagCommand) -> Result<(), Error> {
        let url = self.url("/tag_cmd");
        let body = self
            .http
            .post(&url)
            .form(&[("mac", mac.to_string()), ("cmd", cmd.as_str().to_string())])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        self.check_response_body(&body)
    }

    /// Update a tag's display configuration.
    pub async fn save_tag_config(
        &self,
        mac: &Mac,
        config: &SaveTagConfig,
    ) -> Result<(), Error> {
        let url = self.url("/save_cfg");
        let mut params = vec![("mac".to_string(), mac.to_string())];

        if let Some(mode) = config.content_mode {
            params.push(("contentmode".to_string(), mode.to_u8().to_string()));
        }
        if let Some(ref alias) = config.alias {
            params.push(("alias".to_string(), alias.clone()));
        }
        if let Some(ref json) = config.modecfgjson {
            params.push(("modecfgjson".to_string(), json.clone()));
        }
        if let Some(rotate) = config.rotate {
            params.push(("rotate".to_string(), rotate.to_u8().to_string()));
        }
        if let Some(lut) = config.lut {
            params.push(("lut".to_string(), lut.to_u8().to_string()));
        }
        if let Some(invert) = config.invert {
            params.push(("invert".to_string(), u8::from(invert).to_string()));
        }

        let body = self
            .http
            .post(&url)
            .form(&params)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        self.check_response_body(&body)
    }

    /// Download the tag database as raw JSON bytes.
    pub async fn backup_db(&self) -> Result<Vec<u8>, Error> {
        let url = self.url("/backup_db");
        let bytes = self.http.get(&url).send().await?.error_for_status()?.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Restore a previously downloaded tag database.
    pub async fn restore_db(&self, data: Vec<u8>) -> Result<(), Error> {
        let url = self.url("/restore_db");
        let part = reqwest::multipart::Part::bytes(data)
            .file_name("tagDB.json")
            .mime_str("application/json")
            .expect("application/json is a valid MIME type");
        let form = reqwest::multipart::Form::new().part("file", part);

        let body = self
            .http
            .post(&url)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        self.check_response_body(&body)
    }

    pub(crate) async fn get_tags_page(&self, pos: u32) -> Result<TagDatabasePage, Error> {
        let pos_str = pos.to_string();
        let url = self.url_with_params("/get_db", &[("pos", &pos_str)]);
        let page: TagDatabasePage = self.http.get(&url).send().await?.error_for_status()?.json().await?;
        Ok(page)
    }
}
