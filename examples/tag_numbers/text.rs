//! Maximised single-line text rendering.
//!
//! Lays out a run of differently-coloured text fragments on one shared baseline
//! and scales the whole line as large as it will go inside a target image, then
//! rasterises it. "As large as it will go" is measured against the glyphs'
//! actual ink box (the union of their rasterised pixel bounds), not the font's
//! nominal ascent/descent, so the visible text genuinely fills the frame.

use ab_glyph::{Font, FontVec, Glyph, GlyphId, Rect, ScaleFont, point};
use image::{Rgb, RgbImage};

/// A contiguous fragment of text sharing a single colour.
pub struct TextRun {
    /// The fragment's characters.
    pub text: String,
    /// RGB colour applied to every glyph in the fragment.
    pub color: [u8; 3],
}

// A reference scale used only for measurement. Glyph outlines scale linearly, so
// the final scale is this reference multiplied by the fit ratio. The value just
// needs to be large enough to keep the measured ink box numerically stable.
const MEASURE_SCALE: f32 = 256.0;

// Anti-aliased glyph edges produce partial coverage. These labels are uploaded
// with the AP's own dithering disabled, so intermediate greys would be
// colour-matched to the palette unpredictably along every edge. Snapping at the
// half-coverage mark keeps the output strictly in the run colours.
const COVERAGE_THRESHOLD: f32 = 0.5;

/// Draw `runs` as one line, scaled to fill `img` minus a `margin`-pixel border
/// on every side, centred both horizontally and vertically. Pixels already in
/// `img` show through wherever a glyph does not cover them, so the caller's
/// background fill becomes the label background.
pub fn draw_maxed_line(img: &mut RgbImage, font: &FontVec, runs: &[TextRun], margin: u32) {
    let avail_w = img.width().saturating_sub(2 * margin) as f32;
    let avail_h = img.height().saturating_sub(2 * margin) as f32;

    // Measure once at the reference scale, then derive the scale that just fits
    // the available box in whichever axis is the binding constraint.
    let measured = layout(font, runs, MEASURE_SCALE);
    let Some(ink) = ink_bounds(font, &measured) else {
        return;
    };
    let fit = (avail_w / ink.width()).min(avail_h / ink.height());
    let scale = MEASURE_SCALE * fit;

    // Re-layout at the final scale and centre its ink box within the full image.
    let glyphs = layout(font, runs, scale);
    let Some(ink) = ink_bounds(font, &glyphs) else {
        return;
    };
    let dx = (img.width() as f32 - ink.width()) / 2.0 - ink.min.x;
    let dy = (img.height() as f32 - ink.height()) / 2.0 - ink.min.y;

    for (glyph, color) in glyphs {
        let color = Rgb(color);
        let Some(outline) = font.outline_glyph(glyph) else {
            continue;
        };
        let bounds = outline.px_bounds();
        let base_x = (bounds.min.x + dx).round() as i32;
        let base_y = (bounds.min.y + dy).round() as i32;
        outline.draw(|gx, gy, coverage| {
            if coverage < COVERAGE_THRESHOLD {
                return;
            }
            let px = base_x + gx as i32;
            let py = base_y + gy as i32;
            if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height() {
                img.put_pixel(px as u32, py as u32, color);
            }
        });
    }
}

/// Place every glyph of every run on a shared baseline at `y = 0`, advancing the
/// pen (with kerning) continuously across run boundaries so a colour change does
/// not disturb spacing. Each positioned glyph is paired with its run colour.
fn layout(font: &FontVec, runs: &[TextRun], scale: f32) -> Vec<(Glyph, [u8; 3])> {
    let scaled = font.as_scaled(scale);
    let mut glyphs = Vec::new();
    let mut pen_x = 0.0;
    let mut previous: Option<GlyphId> = None;
    for run in runs {
        for ch in run.text.chars() {
            let id = font.glyph_id(ch);
            if let Some(prev) = previous {
                pen_x += scaled.kern(prev, id);
            }
            glyphs.push((id.with_scale_and_position(scale, point(pen_x, 0.0)), run.color));
            pen_x += scaled.h_advance(id);
            previous = Some(id);
        }
    }
    glyphs
}

/// Union of every glyph's rasterised ink rectangle, or `None` when nothing has
/// ink (e.g. whitespace only). Coordinates share the layout's baseline-at-zero
/// frame, so this measures the visible extent of the line as a whole.
fn ink_bounds(font: &FontVec, glyphs: &[(Glyph, [u8; 3])]) -> Option<Rect> {
    let mut acc: Option<Rect> = None;
    for (glyph, _) in glyphs {
        let Some(outline) = font.outline_glyph(glyph.clone()) else {
            continue;
        };
        let b = outline.px_bounds();
        acc = Some(match acc {
            None => b,
            Some(cur) => Rect {
                min: point(cur.min.x.min(b.min.x), cur.min.y.min(b.min.y)),
                max: point(cur.max.x.max(b.max.x), cur.max.y.max(b.max.y)),
            },
        });
    }
    acc
}
