use serde::de::Deserializer;
use serde::Deserialize;

use super::ap_item::ApListItem;
use super::system::SystemHeartbeat;
use super::tag::TagRecord;

/// A message received from the AP's WebSocket endpoint.
///
/// Each message is a JSON object with exactly one identifying top-level key.
/// The variant is determined by that key.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WsMessage {
    /// Runtime health metrics (`"sys"` key). Sent ~every 5 seconds.
    SystemInfo(SystemHeartbeat),
    /// Tag status update (`"tags"` key). Sent on check-in or config change.
    TagUpdate(Vec<TagRecord>),
    /// Discovered remote access point (`"apitem"` key).
    ApItem(ApListItem),
    /// General log message (`"logMsg"` key).
    Log(String),
    /// Error or critical status message (`"errMsg"` key).
    Error(String),
    /// Serial/progress output (`"console"` key), with optional color.
    Console {
        /// The output text.
        text: String,
        /// Optional CSS color string.
        color: Option<String>,
    },
    /// A message type not recognized by this SDK version.
    Unknown {
        /// The top-level JSON key.
        key: String,
        /// The full message as a raw JSON value.
        raw: serde_json::Value,
    },
}

// The AP's WebSocket messages cannot use serde's standard enum tagging
// because: (1) the "console" message has two sibling top-level keys
// ("console" + "color"), and (2) the Unknown fallback needs to capture
// both the key name and the raw payload.

impl<'de> Deserialize<'de> for WsMessage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map: serde_json::Map<String, serde_json::Value> =
            serde_json::Map::deserialize(deserializer)?;

        if let Some(value) = map.get("sys") {
            let heartbeat: SystemHeartbeat =
                serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;
            return Ok(WsMessage::SystemInfo(heartbeat));
        }

        if let Some(value) = map.get("tags") {
            let tags: Vec<TagRecord> =
                serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;
            return Ok(WsMessage::TagUpdate(tags));
        }

        if let Some(value) = map.get("apitem") {
            let item: ApListItem =
                serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;
            return Ok(WsMessage::ApItem(item));
        }

        if let Some(value) = map.get("logMsg") {
            let msg = value.as_str().unwrap_or_default().to_string();
            return Ok(WsMessage::Log(msg));
        }

        if let Some(value) = map.get("errMsg") {
            let msg = value.as_str().unwrap_or_default().to_string();
            return Ok(WsMessage::Error(msg));
        }

        if let Some(value) = map.get("console") {
            let text = value.as_str().unwrap_or_default().to_string();
            let color = map.get("color").and_then(|v| v.as_str()).map(String::from);
            return Ok(WsMessage::Console { text, color });
        }

        if let Some((key, _)) = map.iter().next() {
            return Ok(WsMessage::Unknown {
                key: key.clone(),
                raw: serde_json::Value::Object(map),
            });
        }

        Err(serde::de::Error::custom("empty WebSocket message"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_log_message() {
        let json = r#"{"logMsg": "00007E23907FB299 update sent"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Log(text) => assert_eq!(text, "00007E23907FB299 update sent"),
            other => panic!("expected Log, got {other:?}"),
        }
    }

    #[test]
    fn parse_error_message() {
        let json = r#"{"errMsg": "REBOOTING"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Error(text) => assert_eq!(text, "REBOOTING"),
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn parse_console_with_color() {
        let json = r#"{"console": "Flashing succeeded", "color": "green"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Console { text, color } => {
                assert_eq!(text, "Flashing succeeded");
                assert_eq!(color.as_deref(), Some("green"));
            }
            other => panic!("expected Console, got {other:?}"),
        }
    }

    #[test]
    fn parse_console_without_color() {
        let json = r#"{"console": "Progress: 45%"}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Console { text, color } => {
                assert_eq!(text, "Progress: 45%");
                assert!(color.is_none());
            }
            other => panic!("expected Console, got {other:?}"),
        }
    }

    #[test]
    fn parse_system_heartbeat() {
        let json = r#"{"sys":{"currtime":1780232927,"heap":245760,"recordcount":3,"dbsize":98304,"littlefsfree":1048576,"apstate":1,"runstate":2,"rssi":-55,"wifistatus":3,"wifissid":"TestNet","uptime":86400}}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::SystemInfo(hb) => {
                assert_eq!(hb.currtime, 1780232927);
                assert_eq!(hb.uptime, 86400);
            }
            other => panic!("expected SystemInfo, got {other:?}"),
        }
    }

    #[test]
    fn parse_ap_item() {
        let json = r#"{"apitem":{"ip":"192.168.1.100","alias":"Remote","count":15,"channel":"11","version":"001F"}}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::ApItem(item) => {
                assert_eq!(item.ip, "192.168.1.100");
                assert_eq!(item.count, 15);
            }
            other => panic!("expected ApItem, got {other:?}"),
        }
    }

    #[test]
    fn parse_unknown_message() {
        let json = r#"{"futureField": {"data": 42}}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::Unknown { key, .. } => assert_eq!(key, "futureField"),
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn parse_tag_update() {
        let json = r#"{"tags":[{"mac":"00007E23907FB299","hash":"aabb","lastseen":0,"nextupdate":0,"nextcheckin":0,"pending":0,"alias":"","contentMode":0,"LQI":0,"RSSI":0,"temperature":0,"batteryMv":0,"hwType":0,"wakeupReason":0,"capabilities":0,"modecfgjson":"","isexternal":false,"apip":"0.0.0.0","rotate":0,"lut":0,"invert":0,"updatecount":0,"updatelast":0,"ch":0,"ver":0}]}"#;
        let msg: WsMessage = serde_json::from_str(json).unwrap();
        match msg {
            WsMessage::TagUpdate(tags) => assert_eq!(tags.len(), 1),
            other => panic!("expected TagUpdate, got {other:?}"),
        }
    }
}
