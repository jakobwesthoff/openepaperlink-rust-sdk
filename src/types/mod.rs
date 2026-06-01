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
/// Tag type descriptors: display dimensions and color palettes.
mod tag_type;
/// WebSocket message enum.
mod ws_message;

pub use ap_item::ApListItem;
pub use config::{ApConfig, SaveApConfig};
pub use content::UploadImageOptions;
pub use led::LedFlashPattern;
pub use mac::Mac;
pub use system::{ApState, RunState, SystemHeartbeat, SystemInfo};
pub use tag::{
    Battery, ContentMode, LutMode, NextCheckin, Rotation, Rssi, SaveTagConfig, TagCommand,
    TagRecord, WakeupReason,
};
pub use tag_type::{ColorEntry, TagType};
pub use ws_message::WsMessage;

pub(crate) use tag::TagDatabasePage;
