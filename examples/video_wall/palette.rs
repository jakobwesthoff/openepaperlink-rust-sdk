//! Deriving the two working palettes from a tag type.
//!
//! Dithering and uploading use different colors on purpose:
//! - We *dither against* the perceptual palette (what the panel visually shows)
//!   so the result looks faithful.
//! - We *emit* the AP's color keys into the JPEG so that, with the AP's own
//!   dithering disabled, its nearest-color match lands exactly on the intended
//!   color.
//!
//! The two are kept index-aligned by matching on color **name**, never on
//! position (the wire color tables are JSON objects with no guaranteed order).

use openepaperlink_sdk::{ColorEntry, TagType};

/// Index-aligned palettes: `dither[i]` and `emit[i]` are the same logical color.
pub struct Palettes {
    /// Colors to dither against (perceptual when available, else the keys).
    pub dither: Vec<[u8; 3]>,
    /// Colors to write into the uploaded JPEG (the AP's color keys).
    pub emit: Vec<[u8; 3]>,
}

impl Palettes {
    /// Build the dither/emit palettes for a tag type.
    pub fn from_tag_type(tag_type: &TagType) -> Self {
        build_palettes(tag_type.render_palette(), &tag_type.color_table)
    }
}

/// Pair each rendered color with its matching color key by name.
///
/// `render` drives the index order. For each rendered color we look up the key
/// of the same name; if none exists (a perceptual-only color), we fall back to
/// the rendered color's own RGB so the index still has a usable emit value.
fn build_palettes(render: &[ColorEntry], keys: &[ColorEntry]) -> Palettes {
    let dither = render.iter().map(|c| c.rgb).collect();
    let emit = render
        .iter()
        .map(|c| {
            keys.iter()
                .find(|k| k.name == c.name)
                .map_or(c.rgb, |k| k.rgb)
        })
        .collect();
    Palettes { dither, emit }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, rgb: [u8; 3]) -> ColorEntry {
        ColorEntry { name: name.into(), rgb }
    }

    #[test]
    fn dithers_against_perceptual_emits_keys() {
        let keys = vec![
            entry("white", [255, 255, 255]),
            entry("black", [0, 0, 0]),
            entry("red", [255, 0, 0]),
        ];
        let perceptual = vec![
            entry("white", [255, 255, 255]),
            entry("black", [0, 0, 0]),
            entry("red", [200, 30, 30]),
        ];

        let p = build_palettes(&perceptual, &keys);

        // Dither sees the toned-down perceptual red...
        assert_eq!(p.dither[2], [200, 30, 30]);
        // ...but we emit the pure color key.
        assert_eq!(p.emit[2], [255, 0, 0]);
    }

    #[test]
    fn matches_keys_by_name_not_position() {
        // Keys deliberately in a different order than the render palette.
        let keys = vec![entry("red", [255, 0, 0]), entry("black", [0, 0, 0])];
        let render = vec![entry("black", [0, 0, 0]), entry("red", [200, 30, 30])];

        let p = build_palettes(&render, &keys);

        assert_eq!(p.emit, vec![[0, 0, 0], [255, 0, 0]]);
    }

    #[test]
    fn falls_back_to_render_rgb_when_no_key_matches() {
        let keys = vec![entry("black", [0, 0, 0])];
        let render = vec![entry("ghost", [123, 45, 67])];

        let p = build_palettes(&render, &keys);

        assert_eq!(p.emit, vec![[123, 45, 67]]);
    }
}
