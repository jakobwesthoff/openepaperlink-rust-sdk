use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::Mac;

// =========================================================
// Wakeup Reason
// =========================================================

/// Why a tag last woke up from sleep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WakeupReason {
    /// Scheduled timer wakeup (normal check-in cycle).
    Timed,
    /// Normal boot.
    Boot,
    /// GPIO pin triggered wakeup.
    Gpio,
    /// NFC field detected.
    Nfc,
    /// Physical button 1 pressed.
    Button1,
    /// Physical button 2 pressed.
    Button2,
    /// Physical button 3 pressed.
    Button3,
    /// An OTA firmware update was rejected by the tag.
    FailedOtaFirmware,
    /// Very first boot after flashing.
    FirstBoot,
    /// Tag is scanning for access points.
    NetworkScan,
    /// Watchdog timer forced a reset.
    WatchdogReset,
    /// A wakeup reason not recognized by this SDK version.
    Unknown(u8),
}

impl WakeupReason {
    fn from_u8(v: u8) -> Self {
        match v {
            0x00 => Self::Timed,
            0x01 => Self::Boot,
            0x02 => Self::Gpio,
            0x03 => Self::Nfc,
            0x04 => Self::Button1,
            0x05 => Self::Button2,
            0x06 => Self::Button3,
            0xE0 => Self::FailedOtaFirmware,
            0xFC => Self::FirstBoot,
            0xFD => Self::NetworkScan,
            0xFE => Self::WatchdogReset,
            other => Self::Unknown(other),
        }
    }

    fn to_u8(self) -> u8 {
        match self {
            Self::Timed => 0x00,
            Self::Boot => 0x01,
            Self::Gpio => 0x02,
            Self::Nfc => 0x03,
            Self::Button1 => 0x04,
            Self::Button2 => 0x05,
            Self::Button3 => 0x06,
            Self::FailedOtaFirmware => 0xE0,
            Self::FirstBoot => 0xFC,
            Self::NetworkScan => 0xFD,
            Self::WatchdogReset => 0xFE,
            Self::Unknown(v) => v,
        }
    }
}

impl Serialize for WakeupReason {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for WakeupReason {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = u8::deserialize(deserializer)?;
        Ok(Self::from_u8(v))
    }
}

// =========================================================
// Content Mode
// =========================================================

/// What the AP renders for a tag's display.
///
/// Not all modes are available for all tag types — the tag type descriptor's
/// `contentids` array lists which modes a specific hardware type supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContentMode {
    /// Not configured.
    None,
    /// Current date display.
    CurrentDate,
    /// Day counter with threshold.
    CountDays,
    /// Hour counter with threshold.
    CountHours,
    /// Current weather via Open-Meteo.
    CurrentWeather,
    /// OTA firmware update for the tag.
    FirmwareUpdate,
    /// External JPEG image fetched by URL.
    ImageUrl,
    /// 5-day weather forecast.
    WeatherForecast,
    /// RSS feed headlines.
    RssFeed,
    /// Full-screen QR code.
    QrCode,
    /// Google Calendar appointments.
    GoogleCalendar,
    /// Content managed by a different AP.
    RemoteContent,
    /// Segment display control (debug).
    SetSegments,
    /// Program an NFC URL on the tag's chip.
    SetNfcUrl,
    /// Dutch rain predictions.
    Buienradar,
    /// Send a raw command to the tag (development only).
    SendCommand,
    /// Configure tag hardware settings.
    SetTagConfig,
    /// Render from a JSON drawing template.
    JsonTemplate,
    /// Mirror another tag's display.
    DisplayCopy,
    /// Show access point status info.
    ApInfo,
    /// Display a static JPEG from the filesystem.
    StaticImage,
    /// Preload an image for triggered display.
    ImagePreload,
    /// Image from an external source (e.g., imgupload API).
    ExternalImage,
    /// Image rendered by Home Assistant.
    HomeAssistant,
    /// Button press timestamp tracker.
    TimeStamp,
    /// Dynamic electricity tariffs.
    DayaheadPrices,
    /// Reprogram the tag's MAC address.
    SetTagMac,
    /// Live clock display.
    CurrentTime,
    /// Tag was removed by a remote AP.
    RemovedByRemoteAp,
    /// A content mode not recognized by this SDK version.
    Unknown(u8),
}

impl ContentMode {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::None,
            1 => Self::CurrentDate,
            2 => Self::CountDays,
            3 => Self::CountHours,
            4 => Self::CurrentWeather,
            5 => Self::FirmwareUpdate,
            7 => Self::ImageUrl,
            8 => Self::WeatherForecast,
            9 => Self::RssFeed,
            10 => Self::QrCode,
            11 => Self::GoogleCalendar,
            12 => Self::RemoteContent,
            13 => Self::SetSegments,
            14 => Self::SetNfcUrl,
            16 => Self::Buienradar,
            17 => Self::SendCommand,
            18 => Self::SetTagConfig,
            19 => Self::JsonTemplate,
            20 => Self::DisplayCopy,
            21 => Self::ApInfo,
            22 => Self::StaticImage,
            23 => Self::ImagePreload,
            24 => Self::ExternalImage,
            25 => Self::HomeAssistant,
            26 => Self::TimeStamp,
            27 => Self::DayaheadPrices,
            28 => Self::SetTagMac,
            29 => Self::CurrentTime,
            255 => Self::RemovedByRemoteAp,
            other => Self::Unknown(other),
        }
    }

    pub(crate) fn to_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::CurrentDate => 1,
            Self::CountDays => 2,
            Self::CountHours => 3,
            Self::CurrentWeather => 4,
            Self::FirmwareUpdate => 5,
            Self::ImageUrl => 7,
            Self::WeatherForecast => 8,
            Self::RssFeed => 9,
            Self::QrCode => 10,
            Self::GoogleCalendar => 11,
            Self::RemoteContent => 12,
            Self::SetSegments => 13,
            Self::SetNfcUrl => 14,
            Self::Buienradar => 16,
            Self::SendCommand => 17,
            Self::SetTagConfig => 18,
            Self::JsonTemplate => 19,
            Self::DisplayCopy => 20,
            Self::ApInfo => 21,
            Self::StaticImage => 22,
            Self::ImagePreload => 23,
            Self::ExternalImage => 24,
            Self::HomeAssistant => 25,
            Self::TimeStamp => 26,
            Self::DayaheadPrices => 27,
            Self::SetTagMac => 28,
            Self::CurrentTime => 29,
            Self::RemovedByRemoteAp => 255,
            Self::Unknown(v) => v,
        }
    }
}

impl Serialize for ContentMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for ContentMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = u8::deserialize(deserializer)?;
        Ok(Self::from_u8(v))
    }
}

// =========================================================
// Tag Command
// =========================================================

/// A command that can be sent to a tag via `POST /tag_cmd`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TagCommand {
    /// Delete this tag from the AP's database.
    Delete,
    /// Delete all inactive/timed-out tags.
    Purge,
    /// Clear the pending data queue for this tag.
    Clear,
    /// Force content regeneration.
    Refresh,
    /// Reboot the tag.
    Reboot,
    /// Tell the tag to scan for AP channels.
    Scan,
    /// Reset tag settings to factory defaults.
    Reset,
    /// Put the tag into deep sleep.
    DeepSleep,
    /// Flash the tag LED with a default RGB pattern.
    LedFlash,
    /// Flash the tag LED with a long red pattern.
    LedFlashLong,
    /// Stop any active LED flashing.
    LedFlashStop,
}

impl TagCommand {
    /// The wire string sent as the `cmd` form parameter.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Delete => "del",
            Self::Purge => "purge",
            Self::Clear => "clear",
            Self::Refresh => "refresh",
            Self::Reboot => "reboot",
            Self::Scan => "scan",
            Self::Reset => "reset",
            Self::DeepSleep => "deepsleep",
            Self::LedFlash => "ledflash",
            Self::LedFlashLong => "ledflash_long",
            Self::LedFlashStop => "ledflash_stop",
        }
    }
}

// =========================================================
// Battery (sentinel type)
// =========================================================

/// Tag battery voltage reading, with sentinel values decoded.
///
/// The AP uses several magic values: 0 means no reading is available,
/// 1337 indicates a virtual/non-physical tag, and 2600 means the reading
/// is capped at "≥ 2.6V".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Battery {
    /// No battery reading available (wire value: 0).
    NotAvailable,
    /// Virtual or non-physical tag (wire value: 1337).
    Virtual,
    /// Reading capped at ≥ 2600 mV (wire value: 2600).
    AtLeast(u16),
    /// Actual battery voltage in millivolts.
    Exact(u16),
}

impl Serialize for Battery {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let v = match self {
            Self::NotAvailable => 0,
            Self::Virtual => 1337,
            Self::AtLeast(mv) => *mv,
            Self::Exact(mv) => *mv,
        };
        serializer.serialize_u16(v)
    }
}

impl<'de> Deserialize<'de> for Battery {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = u16::deserialize(deserializer)?;
        Ok(match v {
            0 => Self::NotAvailable,
            1337 => Self::Virtual,
            2600 => Self::AtLeast(2600),
            other => Self::Exact(other),
        })
    }
}

// =========================================================
// NextCheckin (sentinel type)
// =========================================================

/// When the tag is expected to next check in with the AP.
///
/// The sentinel value 3216153600 indicates the tag is in deep sleep and
/// will not check in on any schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NextCheckin {
    /// Tag is in deep sleep (wire value: 3216153600).
    DeepSleep,
    /// Expected check-in at this Unix timestamp.
    At(u32),
}

impl Serialize for NextCheckin {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let v = match self {
            Self::DeepSleep => 3216153600,
            Self::At(ts) => *ts,
        };
        serializer.serialize_u32(v)
    }
}

impl<'de> Deserialize<'de> for NextCheckin {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = u32::deserialize(deserializer)?;
        Ok(if v == 3216153600 {
            Self::DeepSleep
        } else {
            Self::At(v)
        })
    }
}

// =========================================================
// RSSI (sentinel type)
// =========================================================

/// Tag signal strength, with a sentinel for "this tag is the AP itself."
///
/// RSSI of exactly 100 means the tag record represents the access point's
/// own virtual tag entry, not a real radio measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rssi {
    /// This tag record represents the AP itself (wire value: 100).
    AccessPoint,
    /// Actual received signal strength in dBm (typically negative).
    Dbm(i8),
}

impl Serialize for Rssi {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let v = match self {
            Self::AccessPoint => 100,
            Self::Dbm(dbm) => *dbm,
        };
        serializer.serialize_i8(v)
    }
}

impl<'de> Deserialize<'de> for Rssi {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = i8::deserialize(deserializer)?;
        Ok(if v == 100 {
            Self::AccessPoint
        } else {
            Self::Dbm(v)
        })
    }
}

// =========================================================
// Rotation
// =========================================================

/// Display rotation applied to the tag's screen content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rotation {
    /// No rotation (0°).
    None,
    /// 90° clockwise.
    Cw90,
    /// 180°.
    Cw180,
    /// 270° clockwise (90° counter-clockwise).
    Cw270,
    /// A rotation value not recognized by this SDK version.
    Unknown(u8),
}

impl Rotation {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::None,
            1 => Self::Cw90,
            2 => Self::Cw180,
            3 => Self::Cw270,
            other => Self::Unknown(other),
        }
    }

    pub(crate) fn to_u8(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Cw90 => 1,
            Self::Cw180 => 2,
            Self::Cw270 => 3,
            Self::Unknown(v) => v,
        }
    }
}

impl Serialize for Rotation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for Rotation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_u8(u8::deserialize(deserializer)?))
    }
}

// =========================================================
// LUT Mode
// =========================================================

/// Display refresh mode (Look-Up Table selection).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LutMode {
    /// Automatic selection (full refresh when needed).
    Auto,
    /// Always perform a full display refresh.
    FullRefresh,
    /// Fast partial refresh without red/yellow colors.
    Fast,
    /// Fastest refresh with visible ghosting artifacts.
    Fastest,
    /// A LUT mode not recognized by this SDK version.
    Unknown(u8),
}

impl LutMode {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Auto,
            1 => Self::FullRefresh,
            2 => Self::Fast,
            3 => Self::Fastest,
            other => Self::Unknown(other),
        }
    }

    pub(crate) fn to_u8(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::FullRefresh => 1,
            Self::Fast => 2,
            Self::Fastest => 3,
            Self::Unknown(v) => v,
        }
    }
}

impl Serialize for LutMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for LutMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from_u8(u8::deserialize(deserializer)?))
    }
}

// =========================================================
// Tag Record
// =========================================================

/// A tag's full status record as returned by the AP.
///
/// Corresponds to a single entry in the `GET /get_db` response or a
/// WebSocket `tags` update. Field names on the wire use a mix of lowercase,
/// camelCase, and uppercase — the serde renames handle the mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRecord {
    /// Tag MAC address.
    pub mac: Mac,
    /// MD5 hash of the current display image (32-char lowercase hex).
    pub hash: String,
    /// Unix timestamp of the tag's last check-in.
    #[serde(rename = "lastseen")]
    pub last_seen: u32,
    /// Unix timestamp when the AP will next generate content for this tag.
    #[serde(rename = "nextupdate")]
    pub next_update: u32,
    /// When the tag is expected to next check in, or deep sleep.
    #[serde(rename = "nextcheckin")]
    pub next_checkin: NextCheckin,
    /// Number of pending data transfers queued for this tag.
    pub pending: u16,
    /// User-assigned display name (empty string if unset).
    pub alias: String,
    /// What the AP renders for this tag's display.
    #[serde(rename = "contentMode")]
    pub content_mode: ContentMode,
    /// Link Quality Indicator (0–255).
    #[serde(rename = "LQI")]
    pub lqi: u8,
    /// Received signal strength.
    #[serde(rename = "RSSI")]
    pub rssi: Rssi,
    /// Tag-reported temperature in °C.
    pub temperature: i8,
    /// Battery voltage with sentinel values decoded.
    #[serde(rename = "batteryMv")]
    pub battery: Battery,
    /// Hardware type ID, maps to a tag type descriptor file.
    #[serde(rename = "hwType")]
    pub hw_type: u8,
    /// Why the tag last woke up from sleep.
    #[serde(rename = "wakeupReason")]
    pub wakeup_reason: WakeupReason,
    /// Hardware capability bitmask.
    pub capabilities: u8,
    /// JSON-encoded content-mode-specific configuration.
    #[serde(rename = "modecfgjson")]
    pub mode_config_json: String,
    /// Whether this tag is managed by a different AP.
    #[serde(rename = "isexternal")]
    pub is_external: bool,
    /// IP address of the managing AP (meaningful when `is_external` is true).
    #[serde(rename = "apip")]
    pub ap_ip: String,
    /// Display rotation.
    pub rotate: Rotation,
    /// Display refresh mode.
    pub lut: LutMode,
    /// Color inversion.
    pub invert: u8,
    /// Total number of successful display updates.
    #[serde(rename = "updatecount")]
    pub update_count: u32,
    /// Unix timestamp of the last successful display update.
    #[serde(rename = "updatelast")]
    pub update_last: u32,
    /// Radio channel the tag is currently using.
    pub ch: u8,
    /// Tag firmware version.
    pub ver: u16,
}

/// Internal pagination response from `GET /get_db`.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TagDatabasePage {
    /// Tag records in this page.
    pub tags: Vec<TagRecord>,
    /// If present, the offset for the next page request. Absent on the last page.
    /// The wire field is intentionally misspelled as "continu" by the AP.
    #[serde(rename = "continu")]
    pub continuation: Option<u32>,
}

/// Configuration to save for a tag via `POST /save_cfg`.
///
/// All fields are optional — only provided fields are updated.
#[derive(Debug, Clone, Default)]
pub struct SaveTagConfig {
    /// Content mode to set.
    pub content_mode: Option<ContentMode>,
    /// Display name.
    pub alias: Option<String>,
    /// JSON-encoded mode-specific configuration.
    pub modecfgjson: Option<String>,
    /// Display rotation.
    pub rotate: Option<Rotation>,
    /// Display refresh mode.
    pub lut: Option<LutMode>,
    /// Color inversion.
    pub invert: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- WakeupReason ---

    #[test]
    fn wakeup_reason_known() {
        let val: WakeupReason = serde_json::from_str("0").unwrap();
        assert_eq!(val, WakeupReason::Timed);

        let val: WakeupReason = serde_json::from_str("252").unwrap(); // 0xFC
        assert_eq!(val, WakeupReason::FirstBoot);
    }

    #[test]
    fn wakeup_reason_unknown() {
        let val: WakeupReason = serde_json::from_str("42").unwrap();
        assert_eq!(val, WakeupReason::Unknown(42));
    }

    #[test]
    fn wakeup_reason_round_trip() {
        let original = WakeupReason::NetworkScan;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "253"); // 0xFD
        let back: WakeupReason = serde_json::from_str(&json).unwrap();
        assert_eq!(back, original);
    }

    // --- ContentMode ---

    #[test]
    fn content_mode_known() {
        let val: ContentMode = serde_json::from_str("4").unwrap();
        assert_eq!(val, ContentMode::CurrentWeather);
    }

    #[test]
    fn content_mode_no_id_6() {
        let val: ContentMode = serde_json::from_str("6").unwrap();
        assert_eq!(val, ContentMode::Unknown(6));
    }

    #[test]
    fn content_mode_removed_by_remote() {
        let val: ContentMode = serde_json::from_str("255").unwrap();
        assert_eq!(val, ContentMode::RemovedByRemoteAp);
    }

    #[test]
    fn content_mode_round_trip() {
        let original = ContentMode::JsonTemplate;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "19");
        let back: ContentMode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, original);
    }

    // --- Battery ---

    #[test]
    fn battery_not_available() {
        let val: Battery = serde_json::from_str("0").unwrap();
        assert_eq!(val, Battery::NotAvailable);
        assert_eq!(serde_json::to_string(&val).unwrap(), "0");
    }

    #[test]
    fn battery_virtual() {
        let val: Battery = serde_json::from_str("1337").unwrap();
        assert_eq!(val, Battery::Virtual);
        assert_eq!(serde_json::to_string(&val).unwrap(), "1337");
    }

    #[test]
    fn battery_at_least() {
        let val: Battery = serde_json::from_str("2600").unwrap();
        assert_eq!(val, Battery::AtLeast(2600));
        assert_eq!(serde_json::to_string(&val).unwrap(), "2600");
    }

    #[test]
    fn battery_exact() {
        let val: Battery = serde_json::from_str("3062").unwrap();
        assert_eq!(val, Battery::Exact(3062));
        assert_eq!(serde_json::to_string(&val).unwrap(), "3062");
    }

    // --- NextCheckin ---

    #[test]
    fn next_checkin_deep_sleep() {
        let val: NextCheckin = serde_json::from_str("3216153600").unwrap();
        assert_eq!(val, NextCheckin::DeepSleep);
        assert_eq!(serde_json::to_string(&val).unwrap(), "3216153600");
    }

    #[test]
    fn next_checkin_timestamp() {
        let val: NextCheckin = serde_json::from_str("1780232976").unwrap();
        assert_eq!(val, NextCheckin::At(1780232976));
    }

    // --- Rssi ---

    #[test]
    fn rssi_access_point() {
        let val: Rssi = serde_json::from_str("100").unwrap();
        assert_eq!(val, Rssi::AccessPoint);
        assert_eq!(serde_json::to_string(&val).unwrap(), "100");
    }

    #[test]
    fn rssi_actual() {
        let val: Rssi = serde_json::from_str("-62").unwrap();
        assert_eq!(val, Rssi::Dbm(-62));
        assert_eq!(serde_json::to_string(&val).unwrap(), "-62");
    }

    // --- TagRecord from real AP fixture ---

    #[test]
    fn tag_record_from_live_ap_response() {
        // Captured from a live OpenEPaperLink AP running firmware 2.85
        let json = r##"{"mac":"00007E23907FB299","hash":"4eaaf64af5f3dcc50000000000000000","lastseen":1780232916,"nextupdate":1780234085,"nextcheckin":1780232976,"pending":0,"alias":"","contentMode":4,"LQI":124,"RSSI":-62,"temperature":29,"batteryMv":3062,"hwType":51,"wakeupReason":0,"capabilities":225,"modecfgjson":"{\"location\":\"Zoutelande\",\"units\":\"0\",\"interval\":\"30\",\"#lat\":\"51.50167\",\"#lon\":\"3.48472\",\"#tz\":\"Europe/Amsterdam\"}","isexternal":false,"apip":"0.0.0.0","rotate":0,"lut":0,"invert":0,"updatecount":44,"updatelast":1780228711,"ch":11,"ver":41}"##;

        let tag: TagRecord = serde_json::from_str(json).unwrap();

        assert_eq!(tag.mac.to_string(), "00007E23907FB299");
        assert_eq!(tag.content_mode, ContentMode::CurrentWeather);
        assert_eq!(tag.battery, Battery::Exact(3062));
        assert_eq!(tag.rssi, Rssi::Dbm(-62));
        assert_eq!(tag.wakeup_reason, WakeupReason::Timed);
        assert_eq!(tag.next_checkin, NextCheckin::At(1780232976));
        assert_eq!(tag.hw_type, 51);
        assert_eq!(tag.ch, 11);
        assert!(!tag.is_external);
    }

    #[test]
    fn tag_record_with_deep_sleep_sentinel() {
        // Tag with nextupdate = deep sleep sentinel and battery = 0 (not available)
        let json = r#"{"mac":"000060260FB0CD34","hash":"00000000000000000000000000000000","lastseen":1780232927,"nextupdate":3216153600,"nextcheckin":1780232987,"pending":0,"alias":"","contentMode":21,"LQI":0,"RSSI":-60,"temperature":0,"batteryMv":0,"hwType":224,"wakeupReason":0,"capabilities":0,"modecfgjson":"{}","isexternal":false,"apip":"0.0.0.0","rotate":0,"lut":0,"invert":0,"updatecount":0,"updatelast":0,"ch":0,"ver":0}"#;

        let tag: TagRecord = serde_json::from_str(json).unwrap();

        assert_eq!(tag.battery, Battery::NotAvailable);
        assert_eq!(tag.content_mode, ContentMode::ApInfo);
        assert_eq!(tag.next_checkin, NextCheckin::At(1780232987));
        // nextupdate is raw u32, not a sentinel type
        assert_eq!(tag.next_update, 3216153600);
    }

    #[test]
    fn tag_database_page_with_continuation() {
        let json = r#"{"tags":[{"mac":"00007E23907FB299","hash":"aabb","lastseen":0,"nextupdate":0,"nextcheckin":0,"pending":0,"alias":"","contentMode":0,"LQI":0,"RSSI":0,"temperature":0,"batteryMv":0,"hwType":0,"wakeupReason":0,"capabilities":0,"modecfgjson":"","isexternal":false,"apip":"0.0.0.0","rotate":0,"lut":0,"invert":0,"updatecount":0,"updatelast":0,"ch":0,"ver":0}],"continu":25}"#;

        let page: TagDatabasePage = serde_json::from_str(json).unwrap();
        assert_eq!(page.tags.len(), 1);
        assert_eq!(page.continuation, Some(25));
    }

    #[test]
    fn tag_database_page_without_continuation() {
        let json = r#"{"tags":[]}"#;
        let page: TagDatabasePage = serde_json::from_str(json).unwrap();
        assert!(page.tags.is_empty());
        assert_eq!(page.continuation, None);
    }
}
