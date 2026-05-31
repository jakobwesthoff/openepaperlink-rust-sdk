use crate::client::Client;
use crate::{ApConfig, Error};

impl Client {
    /// Retrieve the AP's configuration (capability flags + runtime settings).
    pub async fn get_ap_config(&self) -> Result<ApConfig, Error> {
        let url = self.url("/get_ap_config");
        let config: ApConfig = self.http.get(&url).send().await?.json().await?;
        Ok(config)
    }
}
