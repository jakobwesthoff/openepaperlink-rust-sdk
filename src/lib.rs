#![deny(missing_docs)]

//! Rust SDK for communicating with
//! [OpenEPaperLink](https://github.com/OpenEPaperLink/OpenEPaperLink) access
//! points.
//!
//! Provides typed HTTP and WebSocket clients for managing e-paper tags,
//! configuring the AP, uploading images, and subscribing to real-time events.

mod error;

/// Wire-format types for all API requests and responses.
pub mod types;

pub use error::Error;
pub use types::*;
