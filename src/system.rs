use crate::client::Client;
use crate::{Error, SystemInfo};

impl Client {
    /// Retrieve build-time and hardware information.
    pub async fn get_sysinfo(&self) -> Result<SystemInfo, Error> {
        let url = self.url("/sysinfo");
        let info: SystemInfo = self.http.get(&url).send().await?.json().await?;
        Ok(info)
    }

    /// Reboot the access point.
    ///
    /// Returns `Ok(())` if the AP accepted the command. The AP will be
    /// unreachable for several seconds while it restarts. Connection drops
    /// after the response are expected and not treated as errors.
    pub async fn reboot(&self) -> Result<(), Error> {
        let url = self.url("/reboot");
        match self.http.post(&url).send().await {
            Ok(resp) => {
                let body = resp.text().await.unwrap_or_default();
                self.check_response_body(&body)
            }
            // Connection reset after the command was sent is expected
            Err(e) if e.is_connect() || e.is_request() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Set the AP's system clock.
    ///
    /// Useful for time synchronization without internet (e.g., from Home
    /// Assistant). The epoch must be > 1600000000.
    pub async fn set_time(&self, epoch: u64) -> Result<(), Error> {
        let url = self.url("/set_time");
        let body = self
            .http
            .post(&url)
            .form(&[("epoch", epoch.to_string())])
            .send()
            .await?
            .text()
            .await?;
        self.check_response_body(&body)
    }
}
