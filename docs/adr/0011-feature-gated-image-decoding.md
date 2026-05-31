# 11. Feature-gated image decoding

Date: 2026-05-31

## Status

Accepted

## Context

The AP serves raw image data that may be zlib-compressed, G5-compressed, or
uncompressed. The pixel data format depends on the tag type descriptor (bpp,
color table, buffer rotation). Callers may want decoded pixel data, or they may
want raw bytes for their own processing.

G5 is a custom 1-bpp codec based on ITU T.6 (Group 4 / MMR fax encoding)
with simplified horizontal mode (fixed-length codes instead of Huffman). The
only existing implementations are:

- `g5decoder.js` in OpenEPaperLink — BSL-licensed (not MIT-compatible)
- `bitbank2/g5_imageconvert` — no license file in repo
- `bitbank2/bb_epaper` — GPLv3 (not MIT-compatible for a library)

A clean-room implementation is required. The underlying G4 algorithm is
specified in ITU T.6 (freely available) and implemented in the `fax` crate
(MIT-licensed, crates.io). The G5 delta on top of G4 is small.

## Decision

Image decoding will be included but behind a cargo feature flag
(`image-decode`), disabled by default. The initial implementation will:

- Support zlib decompression via the `flate2` crate
- Leave G5 decoding as a TODO with a `todo!()` / `unimplemented!()` stub
- Support all pixel format interpretations (1/2/3/4/16 bpp)
- Support buffer rotation based on the tag type descriptor

G5 decoding will be implemented later as a clean-room implementation based on
the ITU T.6 specification and the `fax` crate (MIT) for the G4 core, with the
G5 horizontal mode simplification added on top.

## Consequences

- Default builds have no image-decoding dependencies (`flate2` is only pulled
  in with the feature flag).
- The public API returns raw bytes by default; with the feature flag, a
  decoding function is available.
- G5-compressed images will produce an error until the clean-room
  implementation is complete.
- No BSL-licensed code is ported or referenced during implementation.
