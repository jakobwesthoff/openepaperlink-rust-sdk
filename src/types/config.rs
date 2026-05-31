use serde::{Deserialize, Serialize};

use super::serde_helpers::deserialize_string_bool;
use super::system::ApState;

/// AP configuration as returned by `GET /get_ap_config`.
///
/// Merges compile-time capability flags (string `"0"`/`"1"` on the wire)
/// with runtime settings from the AP's `apconfig.json`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApConfig {
    /// Has ESP32-C6 radio module.
    #[serde(rename = "C6", deserialize_with = "deserialize_string_bool")]
    pub has_c6: bool,
    /// Has ESP32-H2 radio module.
    #[serde(rename = "H2", deserialize_with = "deserialize_string_bool")]
    pub has_h2: bool,
    /// Has TLSR radio module.
    #[serde(rename = "TLSR", deserialize_with = "deserialize_string_bool")]
    pub has_tlsr: bool,
    /// Build with reduced feature set.
    #[serde(deserialize_with = "deserialize_string_bool")]
    pub savespace: bool,
    /// Has external tag flasher hardware.
    #[serde(rename = "hasFlasher", deserialize_with = "deserialize_string_bool")]
    pub has_flasher: bool,
    /// Has BLE writer capability.
    #[serde(rename = "hasBLE", deserialize_with = "deserialize_string_bool")]
    pub has_ble_writer: bool,
    /// Has sub-GHz radio.
    #[serde(rename = "hasSubGhz", deserialize_with = "deserialize_string_bool")]
    pub has_sub_ghz: bool,

    /// Current AP connection state.
    pub apstate: ApState,
    /// Radio channel (0 = auto).
    pub channel: u8,
    /// Sub-GHz channel (0 = disabled).
    #[serde(default)]
    pub subghzchannel: u8,
    /// AP display name.
    pub alias: String,
    /// LED brightness (0–255).
    pub led: u8,
    /// TFT display brightness (0–255).
    pub tft: u8,
    /// Language index.
    pub language: u8,
    /// Maximum tag sleep time in minutes.
    pub maxsleep: u8,
    /// Prevent sleep on tag update.
    pub stopsleep: u8,
    /// Show image previews in web UI.
    pub preview: u8,
    /// Enable nightly reboot at 03:56.
    pub nightlyreboot: u8,
    /// Configuration lock.
    pub lock: u8,
    /// WiFi TX power level.
    pub wifipower: u8,
    /// POSIX timezone string.
    pub timezone: String,
    /// Night mode start hour (0–23).
    pub sleeptime1: u8,
    /// Night mode end hour (0–23).
    pub sleeptime2: u8,
    /// BLE enabled.
    pub ble: u8,
    /// GitHub repository for OTA updates.
    pub repo: String,
    /// PlatformIO build environment name.
    pub env: String,
    /// Discovery enabled.
    pub discovery: u8,
    /// Show timestamps in UI.
    pub showtimestamp: u8,
}

/// Configuration to save via `POST /save_apcfg`.
///
/// All fields are optional — only provided fields are updated on the AP.
#[derive(Debug, Clone, Default)]
pub struct SaveApConfig {
    /// AP display name (max 31 chars).
    pub alias: Option<String>,
    /// Radio channel.
    pub channel: Option<u8>,
    /// Sub-GHz channel.
    pub subghzchannel: Option<u8>,
    /// LED brightness (0–255).
    pub led: Option<u8>,
    /// TFT brightness (0–255).
    pub tft: Option<u8>,
    /// Language index.
    pub language: Option<u8>,
    /// Max tag sleep minutes.
    pub maxsleep: Option<u8>,
    /// Prevent sleep on update.
    pub stopsleep: Option<u8>,
    /// Show image previews.
    pub preview: Option<u8>,
    /// Enable nightly reboot.
    pub nightlyreboot: Option<u8>,
    /// Configuration lock.
    pub lock: Option<u8>,
    /// WiFi TX power.
    pub wifipower: Option<u8>,
    /// POSIX timezone string.
    pub timezone: Option<String>,
    /// Night mode start hour.
    pub sleeptime1: Option<u8>,
    /// Night mode end hour.
    pub sleeptime2: Option<u8>,
    /// BLE enabled.
    pub ble: Option<u8>,
    /// Discovery enabled.
    pub discovery: Option<u8>,
    /// Show timestamps.
    pub showtimestamp: Option<u8>,
    /// GitHub repository for OTA.
    pub repo: Option<String>,
    /// Build environment name.
    pub env: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ap_config_from_live_ap_response() {
        // Captured from a live OpenEPaperLink AP
        let json = r#"{"C6": "1", "H2": "0", "TLSR": "0", "savespace": "0", "hasFlasher": "0", "hasBLE": "1", "hasSubGhz": "0","apstate": "1", "channel": 0, "subghzchannel": 0, "alias": "", "led": 0, "tft": 20, "language": 2, "maxsleep": 0, "stopsleep": 1, "preview": 1, "nightlyreboot": 1, "lock": 0, "wifipower": 34, "timezone": "CET-1CEST-2,M3.5.0/02:00:00,M10.5.0/03:00:00", "sleeptime1": 0, "sleeptime2": 0, "ble": 0, "repo": "OpenEPaperLink/OpenEPaperLink", "env": "ESP32_S3_16_8_YELLOW_AP", "discovery": 0, "showtimestamp": 0}"#;

        let config: ApConfig = serde_json::from_str(json).unwrap();

        assert!(config.has_c6);
        assert!(!config.has_h2);
        assert!(config.has_ble_writer);
        assert_eq!(config.apstate, ApState::Online);
        assert_eq!(config.channel, 0);
        assert_eq!(config.tft, 20);
        assert_eq!(config.wifipower, 34);
        assert_eq!(config.repo, "OpenEPaperLink/OpenEPaperLink");
    }
}
