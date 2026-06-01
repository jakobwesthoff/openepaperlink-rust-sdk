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

mod dither;
mod encode;
mod layout;
mod palette;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use image::imageops::FilterType;
use image::{Rgb, RgbImage, imageops};
use openepaperlink_sdk::{Client, Mac, TagType, UploadImageOptions};

use layout::Grid;
use palette::Palettes;

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

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli).await {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Drive the whole pipeline. Setup failures (bad args, unreachable tags,
/// unreadable image) return `Err` and abort; per-tile *upload* failures are
/// collected and reported at the end without aborting the run.
async fn run(cli: Cli) -> Result<ExitCode, String> {
    let wall = parse_wall(&cli.rows)?;
    let client = Client::builder(&cli.ap_url)
        .build()
        .map_err(|e| format!("building client: {e}"))?;

    // Every tag must share a hardware type so the wall has a single cell size
    // and palette. Fetch the descriptor once for that type.
    let tag_type = fetch_uniform_tag_type(&client, &wall).await?;
    let grid = Grid {
        rows: wall.rows as u32,
        cols: wall.cols as u32,
        cell_width: tag_type.width,
        cell_height: tag_type.height,
        padding: cli.padding,
    };
    let palettes = Palettes::from_tag_type(&tag_type);

    println!(
        "{}×{} wall of {} ({}×{}px each), padding {}px",
        wall.rows, wall.cols, tag_type.name, tag_type.width, tag_type.height, cli.padding,
    );

    // Scale the source to cover the full wall, then dither the whole canvas in
    // one pass so error diffusion is continuous across tile seams.
    let source = image::open(&cli.image)
        .map_err(|e| format!("reading {}: {e}", cli.image.display()))?
        .to_rgb8();
    let canvas = cover_scale(&source, &grid);
    let indices = dither::floyd_steinberg(&canvas, &palettes.dither);
    let emit_canvas = build_emit_canvas(&indices, &palettes.emit, canvas.width(), canvas.height());

    if let Some(ref path) = cli.preview {
        write_preview(&emit_canvas, &grid, path)?;
        println!("preview written to {}", path.display());
    }

    Ok(upload_tiles(&client, &wall, &grid, &emit_canvas).await)
}

/// Parse and validate the `--row` arguments into a rectangular [`WallTags`].
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

/// Confirm all tags share one hardware type and fetch that type's descriptor.
async fn fetch_uniform_tag_type(client: &Client, wall: &WallTags) -> Result<TagType, String> {
    let mut hw_type: Option<u8> = None;

    for mac in wall.macs.iter().flatten() {
        let tag = client
            .get_tag(mac)
            .await
            .map_err(|e| format!("querying tag {mac}: {e}"))?;
        match hw_type {
            None => hw_type = Some(tag.hw_type),
            Some(first) if first != tag.hw_type => {
                return Err(format!(
                    "tags have mixed hardware types (0x{first:02X} and 0x{:02X} on {mac}); \
                     a video wall requires identical tags",
                    tag.hw_type,
                ));
            }
            Some(_) => {}
        }
    }

    let hw_type = hw_type.expect("wall has at least one tag after validation");
    client
        .get_tag_type(hw_type)
        .await
        .map_err(|e| format!("fetching tag type 0x{hw_type:02X}: {e}"))
}

/// Scale `source` to *cover* the wall canvas (fill it, cropping overflow), then
/// center-crop to the exact canvas size.
fn cover_scale(source: &RgbImage, grid: &Grid) -> RgbImage {
    let (canvas_w, canvas_h) = grid.canvas_size();

    // Cover = scale by the larger ratio so both dimensions reach the target.
    let scale = (canvas_w as f64 / source.width() as f64)
        .max(canvas_h as f64 / source.height() as f64);
    let resized_w = ((source.width() as f64 * scale).round() as u32).max(canvas_w);
    let resized_h = ((source.height() as f64 * scale).round() as u32).max(canvas_h);
    let resized = imageops::resize(source, resized_w, resized_h, FilterType::Lanczos3);

    let x = (resized.width() - canvas_w) / 2;
    let y = (resized.height() - canvas_h) / 2;
    imageops::crop_imm(&resized, x, y, canvas_w, canvas_h).to_image()
}

/// Materialize the dithered index buffer into an RGB image using the emit
/// (color-key) palette. Pixel order is row-major, matching the index buffer.
fn build_emit_canvas(indices: &[usize], emit: &[[u8; 3]], width: u32, height: u32) -> RgbImage {
    let mut canvas = RgbImage::new(width, height);
    for (pixel, &index) in canvas.pixels_mut().zip(indices) {
        *pixel = Rgb(emit[index]);
    }
    canvas
}

/// Write a preview PNG with each tile's active area outlined, so the wall
/// layout (and the skipped bezel gaps) is visible. The outline is drawn on a
/// copy; the uploaded tiles are cut from the clean canvas.
fn write_preview(emit_canvas: &RgbImage, grid: &Grid, path: &PathBuf) -> Result<(), String> {
    const OUTLINE: Rgb<u8> = Rgb([255, 0, 255]);

    let mut preview = emit_canvas.clone();
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let rect = grid.tile_rect(row, col);
            for x in rect.x..rect.x + rect.width {
                preview.put_pixel(x, rect.y, OUTLINE);
                preview.put_pixel(x, rect.y + rect.height - 1, OUTLINE);
            }
            for y in rect.y..rect.y + rect.height {
                preview.put_pixel(rect.x, y, OUTLINE);
                preview.put_pixel(rect.x + rect.width - 1, y, OUTLINE);
            }
        }
    }

    preview
        .save(path)
        .map_err(|e| format!("writing preview {}: {e}", path.display()))
}

/// Cut and upload each tile sequentially. Continues past failed uploads,
/// returning a success/failure exit code summarizing the run.
async fn upload_tiles(
    client: &Client,
    wall: &WallTags,
    grid: &Grid,
    emit_canvas: &RgbImage,
) -> ExitCode {
    // Disable the AP's own dithering: the canvas is already dithered, and the
    // tiles carry exact color keys for the AP to match 1:1.
    let options = UploadImageOptions {
        dither: Some(0),
        ..Default::default()
    };

    let mut failures: Vec<Mac> = Vec::new();
    let total = wall.rows * wall.cols;

    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let mac = wall.macs[row as usize][col as usize];
            let rect = grid.tile_rect(row, col);
            let tile = imageops::crop_imm(emit_canvas, rect.x, rect.y, rect.width, rect.height)
                .to_image();
            let jpeg = encode::encode_jpeg_444(tile.as_raw(), rect.width as u16, rect.height as u16);

            match client.upload_image(&mac, jpeg, &options).await {
                Ok(()) => println!("  uploaded {mac} (row {row}, col {col})"),
                Err(e) => {
                    eprintln!("  failed   {mac} (row {row}, col {col}): {e}");
                    failures.push(mac);
                }
            }
        }
    }

    let succeeded = total - failures.len();
    println!("done: {succeeded}/{total} tiles uploaded");
    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        eprintln!("{} tile(s) failed:", failures.len());
        for mac in &failures {
            eprintln!("  {mac}");
        }
        ExitCode::FAILURE
    }
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

    #[test]
    fn cover_scale_fills_exact_canvas_dimensions() {
        // A 10×10 source onto a wide 2×1 wall of 100×50 cells (canvas 200×50).
        let source = RgbImage::from_pixel(10, 10, Rgb([1, 2, 3]));
        let grid = Grid { rows: 1, cols: 2, cell_width: 100, cell_height: 50, padding: 0 };
        let canvas = cover_scale(&source, &grid);
        assert_eq!(canvas.dimensions(), (200, 50));
    }

    #[test]
    fn build_emit_canvas_maps_indices_through_palette() {
        let emit = [[0, 0, 0], [255, 255, 255]];
        // 2×1 image: pixel 0 → black, pixel 1 → white.
        let canvas = build_emit_canvas(&[0, 1], &emit, 2, 1);
        assert_eq!(canvas.get_pixel(0, 0), &Rgb([0, 0, 0]));
        assert_eq!(canvas.get_pixel(1, 0), &Rgb([255, 255, 255]));
    }
}
