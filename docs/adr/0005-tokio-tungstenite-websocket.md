# 5. Use tokio-tungstenite for WebSocket

Date: 2026-05-31

## Status

Accepted

## Context

The AP provides a WebSocket endpoint (`/ws`) for real-time event streaming.
We need a WebSocket client library that works with tokio.

reqwest (our HTTP client) has no built-in WebSocket support — only low-level
`upgrade()` plumbing. The options considered were:

- **`reqwest-websocket`** (0.6.0): third-party crate that extends reqwest with
  a `Stream + Sink` WebSocket API via `async-tungstenite`. ~850k downloads,
  actively maintained but not part of the reqwest project.
- **`tokio-tungstenite`**: direct tokio WebSocket client, part of the core
  tungstenite project. More established, manages the WS connection
  independently from the HTTP client.

Both pull in the tungstenite dependency chain (reqwest itself has no dependency
on tungstenite).

## Decision

We will use tokio-tungstenite for the WebSocket connection.

## Consequences

- The WebSocket connection is managed separately from the HTTP client (reqwest).
  This is a natural fit since the WS connection is long-lived and
  fundamentally different from request/response HTTP.
- tokio-tungstenite is part of the tungstenite project itself, giving it
  stronger maintenance guarantees than a third-party wrapper.
- Adds tungstenite + tokio-tungstenite to the dependency tree. This does not
  overlap with reqwest's dependencies.
