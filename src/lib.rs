#![deny(missing_docs)]

//! Rust SDK for communicating with
//! [OpenEPaperLink](https://github.com/OpenEPaperLink/OpenEPaperLink) access
//! points.
//!
//! Provides typed HTTP and WebSocket clients for managing e-paper tags,
//! configuring the AP, uploading images, and subscribing to real-time events.

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
pub use types::*;
pub use ws::EventStream;

// Re-export StreamExt so callers can use .next() on EventStream without
// adding futures-util as a direct dependency.
pub use futures_util::StreamExt;
