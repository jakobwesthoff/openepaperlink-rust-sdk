//! Video wall: span a single image across a grid of e-paper tags.
//!
//! The wall is described row by row, each row a comma-separated list of tag
//! MACs. All tags must be the same hardware type (uniform size and palette).
//! The image is scaled to *cover* the combined canvas, dithered once against
//! the panel's real palette, then sliced into per-tag tiles and uploaded.
//!
//! ```text
//! video_wall <AP_URL> <IMAGE> --row M,M,M [--row M,M,M ...] [--padding N] [--preview PATH]
//! ```
//!
//! This file currently provides the CLI surface and grid validation; the image
//! pipeline (fetch → scale → dither → cut → upload) is added on top of it.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use openepaperlink_sdk::Mac;

/// Span one image across a grid of e-paper tags as a video wall.
#[derive(Debug, Parser)]
#[command(name = "video_wall")]
struct Cli {
    /// Base URL of the access point, e.g. `http://192.168.1.100`.
    ap_url: String,
    /// Path to the source image (any format the `image` crate can decode).
    image: PathBuf,
    /// One wall row as comma-separated tag MACs. Repeat `--row` per row;
    /// every row must list the same number of tags.
    #[arg(long = "row", required = true, value_name = "MAC,MAC,...")]
    rows: Vec<String>,
    /// Bezel border, in image pixels, skipped around every tag so the image
    /// reads as continuous across the physical gaps between panels.
    #[arg(long, default_value_t = 0)]
    padding: u32,
    /// Optional path to write a PNG preview of the composed, dithered canvas.
    /// Uploads still proceed; this is additive, not a dry run.
    #[arg(long, value_name = "PATH")]
    preview: Option<PathBuf>,
}

/// The wall's tags as a rectangular grid of MACs, row-major.
#[derive(Debug, PartialEq, Eq)]
struct WallTags {
    rows: usize,
    cols: usize,
    /// `rows × cols` MACs, row-major (row 0 first).
    macs: Vec<Vec<Mac>>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let tags = match parse_wall(&cli.rows) {
        Ok(tags) => tags,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "AP {} — {}×{} wall ({} tags), padding {}px, image {}",
        cli.ap_url,
        tags.rows,
        tags.cols,
        tags.rows * tags.cols,
        cli.padding,
        cli.image.display(),
    );
    if let Some(ref path) = cli.preview {
        println!("preview will be written to {}", path.display());
    }

    ExitCode::SUCCESS
}

/// Parse and validate the `--row` arguments into a rectangular [`WallTags`].
///
/// Each entry is one row of comma-separated MACs. Empty cells are rejected, and
/// every row must contain the same number of tags so the wall is rectangular.
fn parse_wall(rows: &[String]) -> Result<WallTags, String> {
    let mut parsed: Vec<Vec<Mac>> = Vec::with_capacity(rows.len());

    for (r, row) in rows.iter().enumerate() {
        let mut macs = Vec::new();
        for cell in row.split(',') {
            let cell = cell.trim();
            if cell.is_empty() {
                return Err(format!("row {} contains an empty tag entry", r + 1));
            }
            let mac: Mac = cell
                .parse()
                .map_err(|e| format!("row {}: invalid MAC {cell:?}: {e}", r + 1))?;
            macs.push(mac);
        }
        parsed.push(macs);
    }

    let cols = parsed[0].len();
    if let Some((r, row)) = parsed.iter().enumerate().find(|(_, row)| row.len() != cols) {
        return Err(format!(
            "row {} has {} tags but row 1 has {cols}; all rows must be equal length",
            r + 1,
            row.len(),
        ));
    }

    Ok(WallTags {
        rows: parsed.len(),
        cols,
        macs: parsed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(specs: &[&str]) -> Vec<String> {
        specs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parses_a_rectangular_wall() {
        let wall = parse_wall(&rows(&[
            "00007E231842B297,00007E231842B298",
            "00007E231842B299,00007E231842B29A",
        ]))
        .unwrap();

        assert_eq!(wall.rows, 2);
        assert_eq!(wall.cols, 2);
        assert_eq!(wall.macs.len(), 2);
        assert_eq!(wall.macs[0].len(), 2);
    }

    #[test]
    fn single_tag_wall_is_valid() {
        let wall = parse_wall(&rows(&["00007E231842B297"])).unwrap();
        assert_eq!((wall.rows, wall.cols), (1, 1));
    }

    #[test]
    fn tolerates_whitespace_around_macs() {
        let wall = parse_wall(&rows(&[" 00007E231842B297 , 00007E231842B298 "])).unwrap();
        assert_eq!(wall.cols, 2);
    }

    #[test]
    fn rejects_ragged_rows() {
        let err = parse_wall(&rows(&[
            "00007E231842B297,00007E231842B298",
            "00007E231842B299",
        ]))
        .unwrap_err();
        assert!(err.contains("equal length"), "got: {err}");
    }

    #[test]
    fn rejects_empty_cell() {
        let err = parse_wall(&rows(&["00007E231842B297,"])).unwrap_err();
        assert!(err.contains("empty"), "got: {err}");
    }

    #[test]
    fn rejects_invalid_mac() {
        let err = parse_wall(&rows(&["not-a-mac"])).unwrap_err();
        assert!(err.contains("invalid MAC"), "got: {err}");
    }
}
