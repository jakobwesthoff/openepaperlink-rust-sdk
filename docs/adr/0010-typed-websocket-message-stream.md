# 10. Typed WebSocket message stream

Date: 2026-05-31

## Status

Accepted

## Context

The AP's WebSocket endpoint sends JSON messages distinguished by their
top-level key (`sys`, `tags`, `logMsg`, `errMsg`, `apitem`, `console`). We
need to decide how the library exposes these to callers.

## Decision

The WebSocket connection will return a `Stream<Item = Result<WsMessage, Error>>`
where `WsMessage` is a typed enum:

```rust
enum WsMessage {
    SystemInfo(SystemHeartbeat),
    TagUpdate(Vec<TagRecord>),
    ApItem(ApListItem),
    Log(String),
    Error(String),
    Console { text: String, color: Option<String> },
}
```

Deserialization from raw JSON into the correct variant happens inside the
library. Callers receive fully typed messages and pattern-match on the variant.

## Consequences

- Callers never touch raw JSON — all parsing is handled internally.
- Adding a new AP message type requires adding a variant to `WsMessage` (a
  breaking change under semver). This is acceptable because new AP message
  types are rare and callers should handle them.
- The `WsMessage` enum should be marked `#[non_exhaustive]` to allow adding
  variants in minor releases without breaking downstream `match` statements.
- Unknown message types (unrecognized top-level keys) are represented as a
  fallback variant or silently dropped, depending on implementation.
