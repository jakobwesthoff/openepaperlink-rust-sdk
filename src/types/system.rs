use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::serde_helpers::deserialize_string_or_int_u8;

// =========================================================
// AP State
// =========================================================

/// The access point's connection state with its radio module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApState {
    /// Radio module is offline, AP is initializing.
    Offline,
    /// Fully operational.
    Online,
    /// Flashing radio module firmware.
    Flashing,
    /// Waiting for the radio module to reset.
    WaitReset,
    /// The AP requires a power cycle to recover.
    RequiredPowerCycle,
    /// Radio module initialization failed.
    Failed,
    /// Radio module is starting up.
    ComingOnline,
    /// AP is running without a radio module.
    NoRadio,
    /// A state value not recognized by this SDK version.
    Unknown(u8),
}

impl ApState {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Offline,
            1 => Self::Online,
            2 => Self::Flashing,
            3 => Self::WaitReset,
            4 => Self::RequiredPowerCycle,
            5 => Self::Failed,
            6 => Self::ComingOnline,
            7 => Self::NoRadio,
            other => Self::Unknown(other),
        }
    }

    fn to_u8(self) -> u8 {
        match self {
            Self::Offline => 0,
            Self::Online => 1,
            Self::Flashing => 2,
            Self::WaitReset => 3,
            Self::RequiredPowerCycle => 4,
            Self::Failed => 5,
            Self::ComingOnline => 6,
            Self::NoRadio => 7,
            Self::Unknown(v) => v,
        }
    }
}

// ApState appears as a string ("1") in get_ap_config and as an integer (1)
// in the WebSocket sys heartbeat. The deserializer delegates to the
// string-or-int helper, then maps the u8 to the enum variant.

impl Serialize for ApState {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for ApState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = deserialize_string_or_int_u8(deserializer)?;
        Ok(Self::from_u8(v))
    }
}

// =========================================================
// Run State
// =========================================================

/// The AP's operational run status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RunState {
    /// Tag processing is stopped.
    Stop,
    /// Tag processing is paused.
    Pause,
    /// Normal operation.
    Run,
    /// AP is initializing.
    Init,
    /// A state value not recognized by this SDK version.
    Unknown(u8),
}

impl RunState {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Stop,
            1 => Self::Pause,
            2 => Self::Run,
            3 => Self::Init,
            other => Self::Unknown(other),
        }
    }

    fn to_u8(self) -> u8 {
        match self {
            Self::Stop => 0,
            Self::Pause => 1,
            Self::Run => 2,
            Self::Init => 3,
            Self::Unknown(v) => v,
        }
    }
}

impl Serialize for RunState {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for RunState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = u8::deserialize(deserializer)?;
        Ok(Self::from_u8(v))
    }
}

// =========================================================
// System Info (from GET /sysinfo)
// =========================================================

/// Build-time and hardware information from `GET /sysinfo`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// AP display name.
    pub alias: String,
    /// PlatformIO build environment.
    pub env: String,
    /// Build timestamp (epoch seconds, encoded as string).
    pub buildtime: String,
    /// Firmware version string.
    pub buildversion: String,
    /// Git commit SHA.
    pub sha: String,
    /// PSRAM size in bytes.
    pub psramsize: u32,
    /// Flash chip size in bytes.
    pub flashsize: u32,
    /// Whether OTA rollback is available.
    pub rollback: bool,
    /// Radio module firmware version.
    pub ap_version: u16,
    /// Has C6 module (0 or 1).
    #[serde(rename = "hasC6")]
    pub has_c6: u8,
    /// Has H2 module (0 or 1).
    #[serde(rename = "hasH2")]
    pub has_h2: u8,
    /// Has TLSR module (0 or 1).
    #[serde(rename = "hasTslr")]
    pub has_tlsr: u8,
    /// Has external flasher (0 or 1).
    #[serde(rename = "hasFlasher")]
    pub has_flasher: u8,
}

// =========================================================
// System Heartbeat (WebSocket "sys" message)
// =========================================================

/// Runtime health metrics sent via WebSocket approximately every 5 seconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHeartbeat {
    /// Current Unix timestamp.
    pub currtime: u32,
    /// Free heap memory in bytes.
    pub heap: u32,
    /// Number of tags in database (cached, refreshed every 30s).
    pub recordcount: u32,
    /// Database memory usage in bytes.
    pub dbsize: u32,
    /// Free filesystem space in bytes (cached, refreshed every 30s).
    pub littlefsfree: u64,
    /// Free PSRAM in bytes. Only present on boards with PSRAM.
    pub psfree: Option<u32>,
    /// AP connection state.
    pub apstate: ApState,
    /// Operational run status.
    pub runstate: RunState,
    /// WiFi RSSI in dBm.
    pub rssi: i32,
    /// WiFi connection status.
    pub wifistatus: u8,
    /// Connected WiFi SSID.
    pub wifissid: String,
    /// System uptime in seconds.
    pub uptime: u64,
    /// Tags with low battery. Only present approximately once per 60 seconds.
    pub lowbattcount: Option<u32>,
    /// Timed-out tags. Only present approximately once per 60 seconds.
    pub timeoutcount: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ap_state_from_integer() {
        let val: ApState = serde_json::from_str("1").unwrap();
        assert_eq!(val, ApState::Online);
    }

    #[test]
    fn ap_state_from_string() {
        let val: ApState = serde_json::from_str("\"1\"").unwrap();
        assert_eq!(val, ApState::Online);
    }

    #[test]
    fn ap_state_unknown() {
        let val: ApState = serde_json::from_str("99").unwrap();
        assert_eq!(val, ApState::Unknown(99));
    }

    #[test]
    fn ap_state_round_trip() {
        let original = ApState::Flashing;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "2");
        // Round-trip from integer form
        let back: ApState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn run_state_known() {
        let val: RunState = serde_json::from_str("2").unwrap();
        assert_eq!(val, RunState::Run);
    }

    #[test]
    fn run_state_unknown() {
        let val: RunState = serde_json::from_str("42").unwrap();
        assert_eq!(val, RunState::Unknown(42));
    }

    #[test]
    fn run_state_round_trip() {
        let original = RunState::Init;
        let json = serde_json::to_string(&original).unwrap();
        let back: RunState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn system_info_from_live_ap_response() {
        let json = r#"{"alias":"","env":"ESP32_S3_16_8_YELLOW_AP","buildtime":"1768750871","buildversion":"2.85","sha":"46c8b73fa0fd141e6a8a5652b1d855a2c1763924","psramsize":8383863,"flashsize":16777216,"rollback":true,"ap_version":31,"hasC6":1,"hasH2":0,"hasTslr":0,"hasFlasher":0}"#;

        let info: SystemInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.buildversion, "2.85");
        assert_eq!(info.has_c6, 1);
        assert!(info.rollback);
        assert_eq!(info.ap_version, 31);
    }

    #[test]
    fn system_heartbeat_without_optional_fields() {
        let json = r#"{"currtime":1780232927,"heap":245760,"recordcount":3,"dbsize":98304,"littlefsfree":1048576,"apstate":1,"runstate":2,"rssi":-55,"wifistatus":3,"wifissid":"TestNetwork","uptime":86400}"#;

        let hb: SystemHeartbeat = serde_json::from_str(json).unwrap();
        assert_eq!(hb.apstate, ApState::Online);
        assert_eq!(hb.runstate, RunState::Run);
        assert_eq!(hb.psfree, None);
        assert_eq!(hb.lowbattcount, None);
        assert_eq!(hb.timeoutcount, None);
    }

    #[test]
    fn system_heartbeat_with_optional_fields() {
        let json = r#"{"currtime":1780232927,"heap":245760,"recordcount":3,"dbsize":98304,"littlefsfree":1048576,"psfree":4194304,"apstate":1,"runstate":2,"rssi":-55,"wifistatus":3,"wifissid":"TestNetwork","uptime":86400,"lowbattcount":2,"timeoutcount":1}"#;

        let hb: SystemHeartbeat = serde_json::from_str(json).unwrap();
        assert_eq!(hb.psfree, Some(4194304));
        assert_eq!(hb.lowbattcount, Some(2));
        assert_eq!(hb.timeoutcount, Some(1));
    }
}
