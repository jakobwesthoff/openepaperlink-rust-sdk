/// Discovered access point on the network.
mod ap_item;
/// AP configuration types.
mod config;
/// Content upload options.
mod content;
/// LED flash pattern.
mod led;
/// MAC address newtype.
mod mac;
pub(crate) mod serde_helpers;
/// System-level types and enumerations.
mod system;
/// Tag-related types: records, enumerations, sentinel types.
mod tag;
/// WebSocket message enum.
mod ws_message;

pub use ap_item::ApListItem;
pub use config::{ApConfig, SaveApConfig};
pub use content::UploadImageOptions;
pub use led::LedFlashPattern;
pub use mac::Mac;
pub use system::{ApState, RunState, SystemHeartbeat, SystemInfo};
pub use tag::{
    Battery, ContentMode, NextCheckin, Rssi, SaveTagConfig, TagCommand, TagRecord, WakeupReason,
};
pub use ws_message::WsMessage;

// Used by Client::get_tags_page() in src/tags.rs.
#[allow(unused_imports)]
pub(crate) use tag::TagDatabasePage;
