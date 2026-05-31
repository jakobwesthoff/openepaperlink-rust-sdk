# 14. Unknown variant handling with data preservation

Date: 2026-05-31

## Status

Accepted

## Context

The AP firmware evolves independently of this SDK. New WebSocket message types
can appear, new content mode IDs are added (e.g., modes 27–29 were added after
the wiki was last updated), and conditional compilation means different AP
builds expose different features. Enums that fail on unknown values would break
clients whenever the AP firmware is updated.

## Decision

All enums that map to wire values will:

1. Be marked `#[non_exhaustive]` to allow adding variants in minor releases
   without breaking downstream `match` statements.
2. Include an `Unknown` variant that preserves the original wire data:
   - `WsMessage::Unknown { key: String, raw: serde_json::Value }` — preserves
     the top-level key and full JSON payload.
   - `ContentMode::Unknown(u8)` — preserves the numeric content mode ID.
   - Other enums (e.g., `WakeupReason`, `ApState`) follow the same pattern
     with `Unknown(u8)`.

This applies consistently across all enums that deserialize from wire data.

## Consequences

- The SDK never fails or silently drops data due to an unrecognized value from
  the AP. Callers can log, forward, or inspect unknown variants.
- `ContentMode::Unknown(u8)` round-trips correctly: a tag configured with a
  future content mode can be read and written back via `save_cfg` without the
  SDK needing to understand it.
- Callers must handle the `Unknown` variant in `match` (or use a wildcard),
  which `#[non_exhaustive]` already requires. This is the correct behavior —
  unknown data should be a conscious decision, not an invisible gap.
