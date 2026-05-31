# Consider typed enums for rotate, lut, and invert fields

`TagRecord.rotate`, `TagRecord.lut`, and `TagRecord.invert` are bare `u8`
fields with well-defined valid value ranges (0–3 for rotate/lut, 0–1 for
invert). These could be typed enums for better ergonomics, matching the
type-safe approach used elsewhere in the API.
