use serde::{Deserialize, Serialize};

/// A discovered access point on the network, received via WebSocket.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApListItem {
    /// IP address of the discovered AP.
    pub ip: String,
    /// AP display name.
    pub alias: String,
    /// Number of tags managed by this AP.
    pub count: u32,
    /// Radio channel (string-encoded).
    pub channel: String,
    /// Firmware version as 4-character hex string.
    pub version: String,
}
