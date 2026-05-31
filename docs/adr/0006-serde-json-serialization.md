# 6. Use serde and serde_json for serialization

Date: 2026-05-31

## Status

Accepted

## Context

All AP communication uses JSON — HTTP responses, WebSocket messages, and
several request bodies. We need typed serialization and deserialization for
every data structure in the protocol.

## Decision

We will use serde with serde_json and derive macros (`Serialize`,
`Deserialize`) on all wire types.

## Consequences

- All request and response types are statically typed and validated at
  deserialization time.
- reqwest's `.json()` method already requires serde, so this adds no new
  transitive dependency.
- Field renaming (e.g., `next_checkin` → `"nextcheckin"`) is handled
  declaratively via `#[serde(rename = "...")]`.
- Optional fields (e.g., `lowbattcount` in the WebSocket heartbeat, which is
  absent from most messages) are modeled as `Option<T>` with
  `#[serde(default)]`.
