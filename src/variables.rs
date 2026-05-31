use std::collections::HashMap;

use crate::client::Client;
use crate::Error;

impl Client {
    /// Set a single variable in the AP's key-value store.
    pub async fn set_var(&self, key: &str, value: &str) -> Result<(), Error> {
        let url = self.url("/set_var");
        let body = self
            .http
            .post(&url)
            .form(&[("key", key), ("val", value)])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        self.check_response_body(&body)
    }

    /// Set multiple variables at once from a map.
    pub async fn set_vars(&self, vars: &HashMap<String, String>) -> Result<(), Error> {
        let url = self.url("/set_vars");
        let json_str = serde_json::to_string(vars).map_err(Error::Json)?;
        let body = self
            .http
            .post(&url)
            .form(&[("json", &json_str)])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        self.check_response_body(&body)
    }
}
