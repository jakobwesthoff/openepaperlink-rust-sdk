//! Floyd–Steinberg dithering against a fixed palette.
//!
//! The whole canvas is dithered in one pass *before* it is sliced into tiles,
//! so the diffused quantization error crosses tile boundaries and the seams
//! between adjacent panels line up. Dithering each tile in isolation would
//! produce visible discontinuities at the joins.

use image::RgbImage;

/// Dither `canvas` against `palette`, returning the chosen palette index for
/// every pixel in row-major order (`y * width + x`).
///
/// Error is diffused in sRGB space, which is simple and good enough here.
/// Returns all-zero indices for an empty palette (nothing to map to).
//
// TODO: diffusing in linear light would be more physically correct, at the
// cost of a gamma decode/encode per pixel.
pub fn floyd_steinberg(canvas: &RgbImage, palette: &[[u8; 3]]) -> Vec<usize> {
    let width = canvas.width() as i64;
    let height = canvas.height() as i64;
    let pixel_count = (width * height) as usize;

    if palette.is_empty() {
        return vec![0; pixel_count];
    }

    // Signed accumulator so diffused error can push a channel beyond [0, 255].
    let mut buffer: Vec<[f32; 3]> = canvas
        .pixels()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();
    let mut indices = vec![0usize; pixel_count];

    // Standard Floyd–Steinberg weights, distributed to the four forward
    // neighbours (the rest of the kernel sums to 16).
    const NEIGHBOURS: [(i64, i64, f32); 4] = [
        (1, 0, 7.0),
        (-1, 1, 3.0),
        (0, 1, 5.0),
        (1, 1, 1.0),
    ];

    for y in 0..height {
        for x in 0..width {
            let here = (y * width + x) as usize;
            let old = buffer[here];

            let chosen = nearest(old, palette);
            indices[here] = chosen;
            let quantized = palette[chosen];

            let error = [
                old[0] - quantized[0] as f32,
                old[1] - quantized[1] as f32,
                old[2] - quantized[2] as f32,
            ];

            for (dx, dy, weight) in NEIGHBOURS {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || nx >= width || ny < 0 || ny >= height {
                    continue;
                }
                let neighbour = (ny * width + nx) as usize;
                for channel in 0..3 {
                    buffer[neighbour][channel] += error[channel] * weight / 16.0;
                }
            }
        }
    }

    indices
}

/// Index of the palette color closest to `rgb` by squared Euclidean distance.
fn nearest(rgb: [f32; 3], palette: &[[u8; 3]]) -> usize {
    palette
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| distance_sq(rgb, **a).total_cmp(&distance_sq(rgb, **b)))
        .map_or(0, |(index, _)| index)
}

/// Squared Euclidean distance between an accumulated pixel and a palette color.
fn distance_sq(a: [f32; 3], b: [u8; 3]) -> f32 {
    (0..3)
        .map(|channel| {
            let delta = a[channel] - b[channel] as f32;
            delta * delta
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: [u8; 3] = [0, 0, 0];
    const WHITE: [u8; 3] = [255, 255, 255];

    #[test]
    fn nearest_picks_the_closer_color() {
        let palette = [BLACK, WHITE];
        assert_eq!(nearest([10.0, 10.0, 10.0], &palette), 0);
        assert_eq!(nearest([240.0, 240.0, 240.0], &palette), 1);
    }

    #[test]
    fn solid_color_maps_uniformly_with_no_residual_error() {
        // Every pixel already equals a palette color, so there is no error to
        // diffuse and every index is that color.
        let canvas = RgbImage::from_pixel(4, 4, image::Rgb(BLACK));
        let indices = floyd_steinberg(&canvas, &[BLACK, WHITE]);
        assert!(indices.iter().all(|&i| i == 0));
    }

    #[test]
    fn diffuses_error_to_the_next_pixel() {
        // Palette black/white. Pixel 0 is mid-grey 128 → white (1) is marginally
        // closer (127 < 128), leaving error 128-255 = -127. Diffusing 7/16 of
        // that to pixel 1 (originally black) drags it well below white's
        // threshold, so it quantizes to black (0).
        let mut canvas = RgbImage::new(2, 1);
        canvas.put_pixel(0, 0, image::Rgb([128, 128, 128]));
        canvas.put_pixel(1, 0, image::Rgb(BLACK));

        let indices = floyd_steinberg(&canvas, &[BLACK, WHITE]);
        assert_eq!(indices, vec![1, 0]);
    }

    #[test]
    fn empty_palette_returns_zeros_without_panicking() {
        let canvas = RgbImage::from_pixel(3, 2, image::Rgb([10, 20, 30]));
        let indices = floyd_steinberg(&canvas, &[]);
        assert_eq!(indices, vec![0; 6]);
    }
}
