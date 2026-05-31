# 2. Single crate structure

Date: 2026-05-31

## Status

Accepted

## Context

The library needs to expose HTTP client functionality, WebSocket streaming,
type definitions for all API objects, and potentially image decoding. We need
to decide whether to organize this as a single library crate or a Cargo
workspace with multiple crates (e.g., `openepaperlink-types`,
`openepaperlink-client`, `openepaperlink-image`).

## Decision

We will use a single library crate (`openepaperlink-sdk`).

## Consequences

- Simpler dependency management and faster iteration during initial development.
- Downstream users pull one crate; they cannot opt out of transitive
  dependencies they do not need (e.g., pulling the HTTP client just to use the
  type definitions).
- If the crate grows large or the dependency tree becomes a problem, we can
  extract sub-crates later without breaking the public API (re-export from the
  root crate).
