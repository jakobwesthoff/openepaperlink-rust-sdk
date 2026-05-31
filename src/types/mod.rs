/// MAC address newtype with hex encoding and byte-order handling.
mod mac;
pub(crate) mod serde_helpers;
/// System-level enumerations: AP state, run state.
mod system;
/// Tag-related types: enumerations, sentinel types, records.
mod tag;

pub use mac::Mac;
pub use system::{ApState, RunState};
pub use tag::{Battery, ContentMode, NextCheckin, Rssi, TagCommand, WakeupReason};
