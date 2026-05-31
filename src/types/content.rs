/// Options for uploading an image via `POST /imgupload`.
///
/// All fields are optional — the AP applies defaults for omitted fields.
#[derive(Debug, Clone, Default)]
pub struct UploadImageOptions {
    /// Dithering mode: 0 = off, 1 = Floyd-Steinberg, 2 = ordered.
    pub dither: Option<u8>,
    /// Update the tag's alias simultaneously.
    pub alias: Option<String>,
    /// Display rotation (0–3).
    pub rotate: Option<u8>,
    /// LUT mode (0–3).
    pub lut: Option<u8>,
    /// Color inversion.
    pub invert: Option<bool>,
    /// Content mode to set (default: 24 = external image).
    pub content_mode: Option<u8>,
    /// Time to live in minutes (0 = no expiry).
    pub ttl: Option<u32>,
}
