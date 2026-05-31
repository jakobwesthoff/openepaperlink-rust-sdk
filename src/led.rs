//! LED flash control: sending patterns and stopping active sequences.

use crate::client::Client;
use crate::{Error, LedFlashPattern, Mac};

impl Client {
    /// Send an LED flash pattern to a tag.
    pub async fn led_flash(&self, mac: &Mac, pattern: &LedFlashPattern) -> Result<(), Error> {
        let url = self.url_with_params(
            "/led_flash",
            &[("mac", &mac.to_string()), ("pattern", &pattern.to_hex())],
        );
        let body = self.http.get(&url).send().await?.error_for_status()?.text().await?;
        self.check_response_body(&body)
    }

    /// Stop any active LED flashing on a tag.
    pub async fn led_flash_stop(&self, mac: &Mac) -> Result<(), Error> {
        self.led_flash(mac, &LedFlashPattern::stop()).await
    }
}
