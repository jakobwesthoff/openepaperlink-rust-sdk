# 7. Use thiserror for error types

Date: 2026-05-31

## Status

Accepted

## Context

As a library crate, we need a public error type that callers can match on to
handle different failure modes (network errors, deserialization failures,
AP-reported errors, WebSocket disconnects).

## Decision

We will use thiserror to derive our error enum.

## Consequences

- The error type is a structured enum with variants for each failure category.
  Callers can pattern-match to handle specific cases.
- `#[from]` attributes on variants provide automatic conversion from
  underlying errors (reqwest, tungstenite, serde_json).
- Minimal dependency — thiserror is a proc-macro crate with no runtime cost.
- anyhow is explicitly not used in the library itself; callers who prefer
  anyhow can convert via the standard `Error` trait.
