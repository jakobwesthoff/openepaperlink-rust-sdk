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
}
