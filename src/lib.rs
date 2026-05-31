#![deny(missing_docs)]

//! Rust SDK for communicating with
//! [OpenEPaperLink](https://github.com/OpenEPaperLink/OpenEPaperLink) (OEPL)
//! access points.
//!
//! Provides typed HTTP and WebSocket clients for managing e-paper tags,
//! configuring the AP, uploading images, and subscribing to real-time events.
//!
//! # Listing tags and uploading an image
//!
//! ```no_run
//! use openepaperlink_sdk::{Client, Mac, UploadImageOptions};
//!
//! # async fn run() -> Result<(), openepaperlink_sdk::Error> {
//! let client = Client::builder("http://192.168.1.100").build()?;
//!
//! // List all tags the AP knows about
//! for tag in client.get_tags().await? {
//!     println!("{} hw=0x{:02X} battery={:?}", tag.mac, tag.hw_type, tag.battery);
//! }
//!
//! // Push a JPEG image to a specific tag
//! let mac: Mac = "00007E231842B297".parse()?;
//! let jpeg = std::fs::read("label.jpg").expect("read image");
//! client.upload_image(&mac, jpeg, &UploadImageOptions::default()).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Streaming real-time events
//!
//! ```no_run
//! use openepaperlink_sdk::{Client, StreamExt, WsMessage};
//!
//! # async fn run() -> Result<(), openepaperlink_sdk::Error> {
//! let client = Client::builder("http://192.168.1.100").build()?;
//! let mut stream = client.connect_ws().await?;
//!
//! while let Some(Ok(msg)) = stream.next().await {
//!     match msg {
//!         WsMessage::TagUpdate(tags) => {
//!             for tag in &tags {
//!                 println!("tag {} checked in", tag.mac);
//!             }
//!         }
//!         WsMessage::SystemInfo(sys) => {
//!             println!("heap={} tags={}", sys.heap, sys.recordcount);
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod config;
mod content;
mod error;
mod led;
mod system;
mod tags;
mod variables;
mod ws;

/// Wire-format types for all API requests and responses.
pub mod types;

pub use client::{Client, ClientBuilder};
pub use error::Error;
pub use types::{
    ApConfig, ApListItem, ApState, Battery, ContentMode, LedFlashPattern, LutMode, Mac,
    NextCheckin, Rotation, Rssi, RunState, SaveApConfig, SaveTagConfig, SystemHeartbeat,
    SystemInfo, TagCommand, TagRecord, UploadImageOptions, WakeupReason, WsMessage,
};
pub use ws::EventStream;

// Re-export StreamExt so callers can use .next() on EventStream without
// adding futures-util as a direct dependency.
pub use futures_util::StreamExt;
