//! Resolving the three label colours from a tag type's palette.
//!
//! Labels use exactly three colours, looked up by name in the tag type's color
//! *keys* (the RGB values the AP matches uploaded pixels against). We emit those
//! keys directly and disable the AP's dithering, so each rendered pixel lands on
//! the intended panel colour with no nearest-colour drift.

use openepaperlink_sdk::{ColorEntry, TagType};

/// The three colours a `Tag #NNN` label is drawn with.
#[derive(Debug)]
pub struct LabelColors {
    /// Frame background. The white color key.
    pub background: [u8; 3],
    /// The `Tag #` prefix. The black color key.
    pub text: [u8; 3],
    /// The number. The red color key.
    pub number: [u8; 3],
}

impl LabelColors {
    /// Resolve the label colours from a tag type's color table.
    ///
    /// Red is mandatory: the number must render red, so a display without a red
    /// key (a plain black/white panel) is rejected rather than silently drawn in
    /// another colour. White and black fall back to pure values if unnamed,
    /// which no real three-colour descriptor omits.
    pub fn from_tag_type(tag_type: &TagType) -> Result<Self, String> {
        Self::from_color_table(&tag_type.color_table, &tag_type.name)
    }

    fn from_color_table(colors: &[ColorEntry], type_name: &str) -> Result<Self, String> {
        let find = |name: &str| {
            colors
                .iter()
                .find(|c| c.name.eq_ignore_ascii_case(name))
                .map(|c| c.rgb)
        };

        let number = find("red").ok_or_else(|| {
            format!(
                "tag type {type_name:?} has no red in its palette; \
                 a red number needs a three-colour (black/white/red) display"
            )
        })?;

        Ok(LabelColors {
            background: find("white").unwrap_or([255, 255, 255]),
            text: find("black").unwrap_or([0, 0, 0]),
            number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, rgb: [u8; 3]) -> ColorEntry {
        ColorEntry {
            name: name.into(),
            rgb,
        }
    }

    #[test]
    fn resolves_three_color_palette_by_name() {
        let colors = [
            entry("white", [255, 255, 255]),
            entry("black", [0, 0, 0]),
            entry("red", [255, 0, 0]),
        ];
        let resolved = LabelColors::from_color_table(&colors, "BWR").unwrap();

        assert_eq!(resolved.background, [255, 255, 255]);
        assert_eq!(resolved.text, [0, 0, 0]);
        assert_eq!(resolved.number, [255, 0, 0]);
    }

    #[test]
    fn matches_color_names_case_insensitively() {
        let colors = [entry("RED", [200, 10, 10])];
        let resolved = LabelColors::from_color_table(&colors, "x").unwrap();
        assert_eq!(resolved.number, [200, 10, 10]);
    }

    #[test]
    fn rejects_palette_without_red() {
        let colors = [entry("white", [255, 255, 255]), entry("black", [0, 0, 0])];
        let err = LabelColors::from_color_table(&colors, "BW only").unwrap_err();
        assert!(err.contains("no red"), "got: {err}");
    }
}
