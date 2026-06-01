//! JPEG encoding for upload.
//!
//! The AP only accepts JPEG, so each tile is encoded here. Two choices are
//! deliberate and load-bearing:
//! - **Quality 100** — minimize coefficient loss.
//! - **4:4:4 (no chroma subsampling)** — our dithering produces hard, single-
//!   pixel color edges. Subsampling (4:2:0/4:2:2) halves color resolution and
//!   would smear those edges, after which the AP's nearest-color match (with
//!   its own dithering off) would pick the wrong color along every boundary.

use jpeg_encoder::{ColorType, Encoder, SamplingFactor};

/// Encode interleaved RGB8 pixel data as a baseline JPEG at quality 100 with no
/// chroma subsampling. `rgb` must hold exactly `width * height * 3` bytes.
pub fn encode_jpeg_444(rgb: &[u8], width: u16, height: u16) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = Encoder::new(&mut out, 100);
    encoder.set_sampling_factor(SamplingFactor::R_4_4_4);
    encoder
        .encode(rgb, width, height, ColorType::Rgb)
        .expect("RGB buffer sized to width×height×3 always encodes");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_a_valid_jpeg_stream() {
        let rgb = vec![200u8; 2 * 2 * 3];
        let jpeg = encode_jpeg_444(&rgb, 2, 2);

        // SOI marker at the start, EOI marker at the end.
        assert_eq!(&jpeg[..2], &[0xFF, 0xD8]);
        assert_eq!(&jpeg[jpeg.len() - 2..], &[0xFF, 0xD9]);
    }

    #[test]
    fn round_trips_dimensions_and_stays_near_solid_color() {
        let red = [220u8, 20, 60];
        let mut rgb = Vec::new();
        for _ in 0..(4 * 3) {
            rgb.extend_from_slice(&red);
        }

        let jpeg = encode_jpeg_444(&rgb, 4, 3);
        let decoded = image::load_from_memory(&jpeg).unwrap().to_rgb8();

        assert_eq!(decoded.dimensions(), (4, 3));
        // A solid color survives quality-100 4:4:4 within a tight tolerance.
        for pixel in decoded.pixels() {
            for channel in 0..3 {
                let delta = pixel[channel] as i32 - red[channel] as i32;
                assert!(delta.abs() <= 4, "channel drifted by {delta}");
            }
        }
    }
}
