# 12. Mac newtype for MAC addresses

Date: 2026-05-31

## Status

Accepted

## Context

The AP protocol encodes MAC addresses as uppercase hexadecimal strings with
reversed byte order. Both 12-character (6-byte) and 16-character (8-byte)
inputs are accepted. The AP always emits 16-character strings. This encoding
is non-obvious and a common source of silent bugs.

## Decision

We will define a `Mac` newtype wrapping `[u8; 8]` that handles all encoding
concerns:

- Parsing from 12- or 16-character hex strings (with byte reversal)
- `Display` / `ToString` formatting as 16-character uppercase hex
- `Serialize` / `Deserialize` as the hex string (matching the wire format)
- `FromStr` for ergonomic construction

Every API method that takes or returns a MAC address will use `Mac`, not
`String`.

## Consequences

- Callers cannot accidentally pass a raw hex string with wrong byte order or
  wrong length — the type system prevents it.
- The byte-reversal logic is implemented once, tested once, and invisible to
  callers.
- Callers who need the raw bytes can access them via a method on `Mac`.
- Slight construction overhead (`"00007E23907FB299".parse::<Mac>()?`) compared
  to passing a plain string, but this is a feature, not a cost.
