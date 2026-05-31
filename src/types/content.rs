use super::tag::{ContentMode, LutMode, Rotation};

/// Options for uploading an image via `POST /imgupload`.
///
/// All fields are optional — the AP applies defaults for omitted fields.
#[derive(Debug, Clone, Default)]
pub struct UploadImageOptions {
    /// Dithering mode: 0 = off, 1 = Floyd-Steinberg, 2 = ordered.
    pub dither: Option<u8>,
    /// Update the tag's alias simultaneously.
    pub alias: Option<String>,
    /// Display rotation.
    pub rotate: Option<Rotation>,
    /// Display refresh mode.
    pub lut: Option<LutMode>,
    /// Color inversion.
    pub invert: Option<bool>,
    /// Content mode to set (default: external image).
    pub content_mode: Option<ContentMode>,
    /// Time to live in minutes (0 = no expiry).
    pub ttl: Option<u32>,
}
