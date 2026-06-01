//! Tag type descriptors: the display hardware definition behind a tag's
//! `hwType`, served at `GET /tagtypes/<HH>.json`.
//!
//! Only the fields needed to render and target a display are modeled here:
//! dimensions, bit depth, buffer rotation, the two color tables, and the list
//! of supported content modes. Serde ignores everything else by default.
//!
//! Intentionally unmodeled descriptor fields:
//! - `zlib_compression` / `g5_compression` — firmware-version thresholds (hex
//!   strings) gating AP→tag radio compression; irrelevant to the HTTP API.
//! - `highlight_color`, `shortlut`, `options`, `usetemplate` — rendering and
//!   capability hints with no consumer in this SDK.
//! - `template` — an untyped, per-hardware layout blob whose shape varies by
//!   content mode. TODO: not worth modeling until a consumer needs it.

use serde::{Deserialize, Deserializer, de};

use super::tag::{ContentMode, Rotation};

// =========================================================
// Color Entry
// =========================================================

/// One named color in a tag type's palette, e.g. `"red"` → `[255, 0, 0]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorEntry {
    /// Color name as used by the AP (`"white"`, `"black"`, `"red"`, …).
    pub name: String,
    /// 8-bit RGB triple.
    pub rgb: [u8; 3],
}

// =========================================================
// Tag Type
// =========================================================

/// Display hardware descriptor for a tag type, from `GET /tagtypes/<HH>.json`.
///
/// A tag's `hwType` byte (see [`TagRecord::hw_type`](super::TagRecord::hw_type))
/// selects which descriptor applies. The descriptor defines the physical
/// display: its pixel dimensions and its available colors.
///
/// Two color tables are exposed and serve different purposes:
/// - [`color_table`](Self::color_table) holds the AP's color *keys* — the RGB
///   values the AP matches incoming image pixels against.
/// - [`perceptual`](Self::perceptual), when present, holds the RGB values the
///   panel *visually renders*, tuned for display. Prefer it when deciding how
///   an image should look; see [`render_palette`](Self::render_palette).
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct TagType {
    /// Descriptor schema version (used by the AP for cache invalidation).
    pub version: u32,
    /// Human-readable tag model name.
    pub name: String,
    /// Display width in pixels.
    pub width: u32,
    /// Display height in pixels.
    pub height: u32,
    /// Bits per pixel (1, 2, 3, 4, or 16).
    pub bpp: u8,
    /// Buffer rotation the firmware applies to rendered content.
    #[serde(rename = "rotatebuffer")]
    pub rotate_buffer: Rotation,
    /// The AP's color keys, parsed from the `colortable` object in wire order.
    #[serde(rename = "colortable", deserialize_with = "deserialize_color_table")]
    pub color_table: Vec<ColorEntry>,
    /// Perceptual (display-tuned) colors, parsed from the `perceptual` object.
    /// Absent for some tag types.
    #[serde(default, deserialize_with = "deserialize_opt_color_table")]
    pub perceptual: Option<Vec<ColorEntry>>,
    /// Content mode IDs this tag type supports, from `contentids`.
    #[serde(rename = "contentids", default)]
    pub content_ids: Vec<ContentMode>,
}

impl TagType {
    /// The palette to render/dither against: [`perceptual`](Self::perceptual)
    /// when available (it matches what the panel actually shows), otherwise
    /// [`color_table`](Self::color_table).
    pub fn render_palette(&self) -> &[ColorEntry] {
        self.perceptual
            .as_deref()
            .unwrap_or(self.color_table.as_slice())
    }
}

// =========================================================
// Color Table Deserialization
// =========================================================

// The wire format encodes a color table as a JSON object mapping names to
// [r, g, b] arrays. We collect it into an ordered Vec via a map visitor, which
// preserves document order regardless of the serde_json map backing (this
// crate does not enable serde_json's `preserve_order`, so deserializing into a
// map type would not preserve wire order). Consumers must not rely on order
// for correctness — match colors by name — but preserving it keeps debug
// output and previews faithful to the source.

struct ColorTableVisitor;

impl<'de> de::Visitor<'de> for ColorTableVisitor {
    type Value = Vec<ColorEntry>;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a map of color names to [r, g, b] arrays")
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut entries = Vec::new();
        while let Some((name, rgb)) = map.next_entry::<String, [u8; 3]>()? {
            entries.push(ColorEntry { name, rgb });
        }
        Ok(entries)
    }
}

fn deserialize_color_table<'de, D>(deserializer: D) -> Result<Vec<ColorEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_map(ColorTableVisitor)
}

// Only invoked when the `perceptual` field is present (an absent field falls
// back to `Default`, i.e. `None`, via `#[serde(default)]`).
fn deserialize_opt_color_table<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<ColorEntry>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(deserialize_color_table(deserializer)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A realistic black/white/red descriptor, modeled on the protocol doc's
    // hwType 0x33 example, with extra unmodeled fields present to prove they
    // are ignored.
    const BWR_DESCRIPTOR: &str = r##"{
        "version": 1,
        "name": "Solum M3 BWR 2.9\"",
        "width": 296,
        "height": 128,
        "rotatebuffer": 1,
        "bpp": 2,
        "colortable": { "white": [255,255,255], "black": [0,0,0], "red": [255,0,0] },
        "perceptual": { "white": [255,255,255], "black": [0,0,0], "red": [200,30,30] },
        "shortlut": 1,
        "zlib_compression": "0100",
        "g5_compression": "0200",
        "highlight_color": 2,
        "options": ["led"],
        "contentids": [22, 1, 2, 19],
        "usetemplate": 0,
        "template": { "1": { "anything": [1,2,3] } }
    }"##;

    #[test]
    fn deserializes_full_descriptor() {
        let tt: TagType = serde_json::from_str(BWR_DESCRIPTOR).unwrap();

        assert_eq!(tt.version, 1);
        assert_eq!(tt.name, "Solum M3 BWR 2.9\"");
        assert_eq!(tt.width, 296);
        assert_eq!(tt.height, 128);
        assert_eq!(tt.bpp, 2);
        assert_eq!(tt.rotate_buffer, Rotation::Cw90);
    }

    #[test]
    fn parses_color_table_in_wire_order() {
        let tt: TagType = serde_json::from_str(BWR_DESCRIPTOR).unwrap();

        assert_eq!(
            tt.color_table,
            vec![
                ColorEntry { name: "white".into(), rgb: [255, 255, 255] },
                ColorEntry { name: "black".into(), rgb: [0, 0, 0] },
                ColorEntry { name: "red".into(), rgb: [255, 0, 0] },
            ]
        );
    }

    #[test]
    fn parses_perceptual_table() {
        let tt: TagType = serde_json::from_str(BWR_DESCRIPTOR).unwrap();
        let perceptual = tt.perceptual.as_ref().expect("perceptual present");

        // Red is toned down perceptually relative to the pure color key.
        assert_eq!(perceptual[2], ColorEntry { name: "red".into(), rgb: [200, 30, 30] });
    }

    #[test]
    fn maps_content_ids_to_typed_modes() {
        let tt: TagType = serde_json::from_str(BWR_DESCRIPTOR).unwrap();
        assert_eq!(
            tt.content_ids,
            vec![
                ContentMode::StaticImage,   // 22
                ContentMode::CurrentDate,   // 1
                ContentMode::CountDays,     // 2
                ContentMode::JsonTemplate,  // 19
            ]
        );
    }

    #[test]
    fn render_palette_prefers_perceptual() {
        let tt: TagType = serde_json::from_str(BWR_DESCRIPTOR).unwrap();
        // The perceptual red [200,30,30] is returned, not the key red [255,0,0].
        assert_eq!(tt.render_palette()[2].rgb, [200, 30, 30]);
    }

    #[test]
    fn render_palette_falls_back_to_color_table() {
        // Same descriptor without a perceptual table.
        let json = r#"{
            "version": 1, "name": "BW only", "width": 152, "height": 152,
            "rotatebuffer": 0, "bpp": 1,
            "colortable": { "white": [255,255,255], "black": [0,0,0] },
            "contentids": [22]
        }"#;
        let tt: TagType = serde_json::from_str(json).unwrap();

        assert!(tt.perceptual.is_none());
        assert_eq!(tt.render_palette().len(), 2);
        assert_eq!(tt.render_palette()[1].rgb, [0, 0, 0]);
    }

    #[test]
    fn tolerates_missing_optional_fields() {
        // No `perceptual`, no `contentids` — both default to empty/None.
        let json = r#"{
            "version": 1, "name": "Minimal", "width": 100, "height": 100,
            "rotatebuffer": 0, "bpp": 1,
            "colortable": { "white": [255,255,255], "black": [0,0,0] }
        }"#;
        let tt: TagType = serde_json::from_str(json).unwrap();

        assert!(tt.perceptual.is_none());
        assert!(tt.content_ids.is_empty());
    }

    #[test]
    fn rejects_rgb_with_wrong_arity() {
        // A color with only two components is malformed.
        let json = r#"{
            "version": 1, "name": "Bad", "width": 100, "height": 100,
            "rotatebuffer": 0, "bpp": 1,
            "colortable": { "white": [255,255] }
        }"#;
        assert!(serde_json::from_str::<TagType>(json).is_err());
    }

    #[test]
    fn rejects_rgb_component_out_of_range() {
        // 300 does not fit in a u8.
        let json = r#"{
            "version": 1, "name": "Bad", "width": 100, "height": 100,
            "rotatebuffer": 0, "bpp": 1,
            "colortable": { "white": [300,0,0] }
        }"#;
        assert!(serde_json::from_str::<TagType>(json).is_err());
    }

    #[test]
    fn empty_color_table_parses_to_empty_vec() {
        let json = r#"{
            "version": 1, "name": "Empty", "width": 1, "height": 1,
            "rotatebuffer": 0, "bpp": 1, "colortable": {}
        }"#;
        let tt: TagType = serde_json::from_str(json).unwrap();
        assert!(tt.color_table.is_empty());
    }
}
