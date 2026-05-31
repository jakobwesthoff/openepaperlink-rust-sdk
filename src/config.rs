use crate::client::Client;
use crate::{ApConfig, Error, SaveApConfig};

impl Client {
    /// Retrieve the AP's configuration (capability flags + runtime settings).
    pub async fn get_ap_config(&self) -> Result<ApConfig, Error> {
        let url = self.url("/get_ap_config");
        let config: ApConfig = self.http.get(&url).send().await?.error_for_status()?.json().await?;
        Ok(config)
    }

    /// Update the AP's runtime configuration.
    ///
    /// Only fields that are `Some` will be sent. Omitted fields keep their
    /// current values on the AP.
    pub async fn save_ap_config(&self, config: &SaveApConfig) -> Result<(), Error> {
        let url = self.url("/save_apcfg");
        let mut params: Vec<(String, String)> = Vec::new();

        macro_rules! push_opt {
            ($field:ident, $key:expr) => {
                if let Some(ref v) = config.$field {
                    params.push(($key.to_string(), v.to_string()));
                }
            };
        }

        push_opt!(alias, "alias");
        push_opt!(channel, "channel");
        push_opt!(subghzchannel, "subghzchannel");
        push_opt!(led, "led");
        push_opt!(tft, "tft");
        push_opt!(language, "language");
        push_opt!(maxsleep, "maxsleep");
        push_opt!(stopsleep, "stopsleep");
        push_opt!(preview, "preview");
        push_opt!(nightlyreboot, "nightlyreboot");
        push_opt!(lock, "lock");
        push_opt!(wifipower, "wifipower");
        push_opt!(timezone, "timezone");
        push_opt!(sleeptime1, "sleeptime1");
        push_opt!(sleeptime2, "sleeptime2");
        push_opt!(ble, "ble");
        push_opt!(discovery, "discovery");
        push_opt!(showtimestamp, "showtimestamp");
        push_opt!(repo, "repo");
        push_opt!(env, "env");

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
}
