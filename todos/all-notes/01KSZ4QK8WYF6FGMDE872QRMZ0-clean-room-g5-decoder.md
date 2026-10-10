# Clean-room G5 decoder implementation

## Summary

Implement G5 image decompression for the `image-decode` feature without using
any BSL-licensed source code.

## Background

G5 is a custom 1-bpp image codec used by OpenEPaperLink tags. It is based on
ITU T.6 (CCITT Group 4 / MMR fax encoding) with one key simplification: the
horizontal mode uses fixed-length codes instead of Huffman encoding. The
fixed-length code size is determined by the image width
(`ceil(log2(width))` bits).

The existing implementations are all license-incompatible with MIT:
- `g5decoder.js` in OpenEPaperLink — BSL (Business Source License)
- `bitbank2/g5_imageconvert` — no license file
- `bitbank2/bb_epaper` — GPLv3

## Implementation approach

1. Use the ITU T.6 specification (freely available at
   https://www.itu.int/rec/T-REC-T.6-198811-I/en) as the primary reference for
   the Group 4 decoding algorithm.

2. Consider using the `fax` crate (MIT, crates.io) for the standard G4 core,
   then adapting only the horizontal mode to use fixed-length codes instead of
   Huffman tables.

3. Reference `bitbank2/TIFF_G4` (Apache 2.0,
   https://github.com/bitbank2/TIFF_G4) as an additional clean reference for
   the G4 algorithm if needed.

4. Useful secondary reference: Shreevatsa's annotated CCITTFaxDecode
   walkthrough at https://shreevatsa.github.io/site/ccitt.html

## G5 data format

The compressed data is preceded by a header:
- Byte 0: header size (uint8)
- Bytes 1-2: width (uint16 LE)
- Bytes 3-4: height (uint16 LE)
- Byte 5: bpp mode (0/1 = normal, 2 = doubled height for two planes)

After the header, the data is G5-encoded scanlines. The decoder produces 1-bpp
packed output (8 pixels per byte, MSB first, white = 1, black = 0).

## Acceptance criteria

- Decodes G5 data produced by OpenEPaperLink AP firmware
- No code translated or derived from BSL/GPL-licensed sources
- Covered by tests using sample data captured from a live AP
