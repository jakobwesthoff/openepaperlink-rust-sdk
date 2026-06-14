//! Tag numbers: stamp a large `Tag #NNN` identification label onto every tag of
//! a chosen hardware type.
//!
//! The hardware type is named explicitly on the command line (e.g. `'M3 2.9"'`)
//! so there is no guessing which displays are addressed. Every tag of that type
//! gets a full-frame label: `Tag #` in black and a zero-padded sequential
//! number in red, scaled as large as the frame allows. The numbers run `001`,
//! `002`, … in the order the AP lists the tags.
//!
//! ```text
//! tag_numbers <AP_URL> <TAG_TYPE> --font <PATH>
//! ```

mod encode;
mod palette;
mod text;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use ab_glyph::FontVec;
use clap::Parser;
use image::{Rgb, RgbImage};
use openepaperlink_sdk::{Client, TagRecord, TagType, UploadImageOptions};

use palette::LabelColors;
use text::TextRun;

/// Stamp a numbered identification label onto every tag of one hardware type.
#[derive(Debug, Parser)]
#[command(name = "tag_numbers")]
struct Cli {
    /// Base URL of the access point, e.g. `http://192.168.1.100`.
    ap_url: String,
    /// Tag type selector. Every whitespace-separated term must appear in the
    /// tag type's name (case-insensitive), so `'M3 2.9"'` selects
    /// `Solum M3 BWR 2.9"`. Must resolve to exactly one type.
    tag_type: String,
    /// Path to a TrueType/OpenType font used to render the labels.
    #[arg(long, value_name = "PATH")]
    font: PathBuf,
    /// Border, in pixels, kept clear around the label on every side, so the
    /// maximised text does not bleed into the panel edge.
    #[arg(long, default_value_t = 8)]
    padding: u32,
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

/// Drive the whole job. Setup failures (bad font, unreachable AP, unresolvable
/// type selector, palette without red) return `Err` and abort before anything
/// is uploaded; per-tag *upload* failures are collected and reported at the end
/// without aborting the run.
async fn run(cli: Cli) -> Result<ExitCode, String> {
    let font = load_font(&cli.font)?;

    let client = Client::builder(&cli.ap_url)
        .build()
        .map_err(|e| format!("building client: {e}"))?;

    let tags = client
        .get_tags()
        .await
        .map_err(|e| format!("listing tags: {e}"))?;
    if tags.is_empty() {
        println!("AP reports no tags.");
        return Ok(ExitCode::SUCCESS);
    }

    // Resolve every distinct hardware type present (once each), then pick the one
    // the selector names. Colours come from that type's palette.
    let types = fetch_tag_types(&client, &tags).await?;
    let (hw_types, tag_type) = select_tag_type(&cli.tag_type, &types)?;
    let colors = LabelColors::from_tag_type(tag_type)?;

    let targets: Vec<&TagRecord> = tags
        .iter()
        .filter(|t| hw_types.contains(&t.hw_type))
        .collect();
    println!(
        "Labelling {} {:?} tag(s) at {}×{}px",
        targets.len(),
        tag_type.name,
        tag_type.width,
        tag_type.height,
    );

    Ok(upload_labels(&client, &targets, tag_type, &colors, &font, cli.padding).await)
}

/// Load and parse the label font.
fn load_font(path: &PathBuf) -> Result<FontVec, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("reading font {}: {e}", path.display()))?;
    FontVec::try_from_vec(bytes).map_err(|e| format!("parsing font {}: {e}", path.display()))
}

/// Fetch the [`TagType`] descriptor for each distinct `hwType` among `tags`,
/// keyed by `hwType`. Fetched once per type, not once per tag.
async fn fetch_tag_types(
    client: &Client,
    tags: &[TagRecord],
) -> Result<BTreeMap<u8, TagType>, String> {
    let mut types = BTreeMap::new();
    for tag in tags {
        if types.contains_key(&tag.hw_type) {
            continue;
        }
        let descriptor = client
            .get_tag_type(tag.hw_type)
            .await
            .map_err(|e| format!("fetching tag type 0x{:02X}: {e}", tag.hw_type))?;
        types.insert(tag.hw_type, descriptor);
    }
    Ok(types)
}

/// Resolve the selector to the hardware types sharing the single matching name,
/// returning those `hwType`s and the descriptor to render against.
fn select_tag_type<'a>(
    query: &str,
    types: &'a BTreeMap<u8, TagType>,
) -> Result<(Vec<u8>, &'a TagType), String> {
    let candidates: Vec<(u8, &str)> = types.iter().map(|(&hw, t)| (hw, t.name.as_str())).collect();
    let hw_types = resolve_selection(query, &candidates)?;
    let descriptor = types
        .get(&hw_types[0])
        .expect("resolved hw type comes from the map");
    Ok((hw_types, descriptor))
}

/// Core selector logic over `(hwType, name)` pairs. The selector must match
/// exactly one distinct type *name*; zero or several are errors that list the
/// known or matched names so the caller can correct the spelling.
fn resolve_selection(query: &str, candidates: &[(u8, &str)]) -> Result<Vec<u8>, String> {
    if query.split_whitespace().next().is_none() {
        return Err("empty tag type selector".to_string());
    }

    let matched: Vec<(u8, &str)> = candidates
        .iter()
        .copied()
        .filter(|(_, name)| name_matches(query, name))
        .collect();

    // A single distinct name means the selector is unambiguous even if several
    // tags (hwTypes) carry it.
    let names = unique_names(matched.iter().map(|(_, n)| *n));
    match names.as_slice() {
        [] => Err(format!(
            "no tag type matches {query:?}. Known types: {}",
            unique_names(candidates.iter().map(|(_, n)| *n)).join(", "),
        )),
        [_one] => Ok(matched.iter().map(|(hw, _)| *hw).collect()),
        many => Err(format!(
            "selector {query:?} is ambiguous; it matches: {}",
            many.join(", "),
        )),
    }
}

/// A type name matches when every whitespace-separated term of the selector
/// appears in it, compared case-insensitively. Order and adjacency don't
/// matter, so `'M3 2.9"'` matches `Solum M3 BWR 2.9"`.
fn name_matches(query: &str, name: &str) -> bool {
    let name = name.to_lowercase();
    query
        .split_whitespace()
        .all(|term| name.contains(&term.to_lowercase()))
}

/// Sorted, de-duplicated names from an iterator.
fn unique_names<'a>(names: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut names: Vec<&str> = names.collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// Render and upload one label per target tag, numbering them `001`, `002`, …
/// in iteration order. Continues past failed uploads, returning a success or
/// failure exit code summarizing the run.
async fn upload_labels(
    client: &Client,
    targets: &[&TagRecord],
    tag_type: &TagType,
    colors: &LabelColors,
    font: &FontVec,
    padding: u32,
) -> ExitCode {
    // The label already carries exact color keys, so the AP must match them 1:1
    // rather than dithering.
    let options = UploadImageOptions {
        dither: Some(0),
        ..Default::default()
    };

    let mut failures = Vec::new();
    for (index, tag) in targets.iter().enumerate() {
        let number = (index + 1) as u32;
        let label = render_label(tag_type, colors, number, font, padding);
        let jpeg = encode::encode_jpeg_444(
            label.as_raw(),
            label.width() as u16,
            label.height() as u16,
        );

        match client.upload_image(&tag.mac, jpeg, &options).await {
            Ok(()) => println!("  uploaded Tag #{number:03} -> {}", tag.mac),
            Err(e) => {
                eprintln!("  failed   Tag #{number:03} -> {}: {e}", tag.mac);
                failures.push(tag.mac);
            }
        }
    }

    let total = targets.len();
    let succeeded = total - failures.len();
    println!("done: {succeeded}/{total} labels uploaded");
    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        eprintln!("{} label(s) failed:", failures.len());
        for mac in &failures {
            eprintln!("  {mac}");
        }
        ExitCode::FAILURE
    }
}

/// Render a single full-frame label for `number` at the tag type's native
/// resolution: a white frame with `Tag #` (black) and the zero-padded number
/// (red) maximised inside a `padding`-pixel border.
fn render_label(
    tag_type: &TagType,
    colors: &LabelColors,
    number: u32,
    font: &FontVec,
    padding: u32,
) -> RgbImage {
    let mut img = RgbImage::from_pixel(tag_type.width, tag_type.height, Rgb(colors.background));
    text::draw_maxed_line(&mut img, font, &label_runs(number, colors), padding);
    img
}

/// Build the coloured fragments for one label: a black `Tag #` prefix and the
/// zero-padded number in red.
fn label_runs(number: u32, colors: &LabelColors) -> Vec<TextRun> {
    vec![
        TextRun {
            text: "Tag #".to_string(),
            color: colors.text,
        },
        TextRun {
            text: format!("{number:03}"),
            color: colors.number,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_match_ignores_term_order_and_adjacency() {
        assert!(name_matches("M3 2.9\"", "Solum M3 BWR 2.9\""));
        assert!(name_matches("2.9\" m3", "Solum M3 BWR 2.9\""));
    }

    #[test]
    fn name_match_is_case_insensitive() {
        assert!(name_matches("solum", "Solum M3 BWR 2.9\""));
    }

    #[test]
    fn name_match_requires_every_term() {
        assert!(!name_matches("M3 4.2\"", "Solum M3 BWR 2.9\""));
    }

    #[test]
    fn resolve_picks_all_hwtypes_of_the_one_matching_name() {
        // Two hwTypes share a name (e.g. a hardware revision); both are selected.
        let candidates = [(0x01, "M3 2.9\""), (0x02, "M3 2.9\""), (0x10, "M2 1.5\"")];
        let hw = resolve_selection("M3 2.9\"", &candidates).unwrap();
        assert_eq!(hw, vec![0x01, 0x02]);
    }

    #[test]
    fn resolve_errors_when_nothing_matches() {
        let candidates = [(0x01, "M3 2.9\"")];
        let err = resolve_selection("7.5\"", &candidates).unwrap_err();
        assert!(err.contains("no tag type matches"), "got: {err}");
    }

    #[test]
    fn resolve_errors_on_ambiguous_selector() {
        let candidates = [(0x01, "M3 2.9\""), (0x02, "M3 4.2\"")];
        let err = resolve_selection("M3", &candidates).unwrap_err();
        assert!(err.contains("ambiguous"), "got: {err}");
    }

    #[test]
    fn label_runs_split_prefix_and_zero_padded_number() {
        let colors = LabelColors {
            background: [255, 255, 255],
            text: [0, 0, 0],
            number: [255, 0, 0],
        };
        let runs = label_runs(1, &colors);

        assert_eq!(runs[0].text, "Tag #");
        assert_eq!(runs[0].color, [0, 0, 0]);
        assert_eq!(runs[1].text, "001");
        assert_eq!(runs[1].color, [255, 0, 0]);
    }
}
