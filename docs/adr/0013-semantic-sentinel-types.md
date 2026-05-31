# 13. Semantic types for sentinel-bearing fields

Date: 2026-05-31

## Status

Accepted

## Context

Several fields in the tag record carry magic sentinel values alongside real
data:

- `batteryMv`: 0 = not available, 1337 = virtual tag, 2600 = capped "≥ 2.6V",
  other values = actual millivolts
- `nextcheckin`: 3216153600 = deep sleep, other values = Unix timestamp
- `RSSI`: 100 = tag is the AP itself, other values = actual dBm

Exposing these as raw integers forces every caller to know and check the
sentinel values, leading to bugs when they don't.

## Decision

Fields with sentinel values will be modeled as semantic enums with custom
serde deserialization that maps the wire values:

```rust
pub enum Battery {
    NotAvailable,
    Virtual,
    AtLeast(u16),
    Exact(u16),
}

pub enum NextCheckin {
    DeepSleep,
    At(u32),
}

pub enum Rssi {
    AccessPoint,
    Dbm(i8),
}
```

The serde `Deserialize` implementation converts the raw integer from JSON into
the appropriate variant. `Serialize` converts back to the wire integer.

## Consequences

- Callers cannot accidentally treat a sentinel value as real data — the type
  system forces them to handle each case.
- Custom serde implementations are needed for each sentinel-bearing field.
  This is a one-time cost.
- The enums should be `#[non_exhaustive]` in case new sentinel values are
  discovered or added by the AP firmware.
- Round-tripping (deserialize then serialize) must preserve the original wire
  value for correctness.
