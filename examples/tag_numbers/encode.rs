//! JPEG encoding for upload.
//!
//! The AP only accepts JPEG. Labels are flat three-colour images with hard,
//! single-pixel edges between background, text, and number, so two choices
//! matter:
//! - **Quality 100** — minimize coefficient loss.
//! - **4:4:4 (no chroma subsampling)** — subsampling would smear the colour
//!   edges, after which the AP's nearest-colour match (its own dithering off)
//!   would pick the wrong colour along every boundary.

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
}
