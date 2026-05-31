# 18. No TLS by default, opt-in via feature flags

Date: 2026-05-31

## Status

Accepted (revised)

## Context

The OpenEPaperLink AP communicates over plain HTTP/WS on the local network.
However, the AP may sit behind a reverse proxy that terminates TLS, requiring
HTTPS/WSS support from the SDK.

Both reqwest and tokio-tungstenite support multiple TLS backends:

- **reqwest 0.13.x:** defaults include `rustls`, `charset`, `http2`,
  `system-proxy`. TLS features: `rustls`, `native-tls`. Non-TLS features
  needed: `json`, `multipart`, `form`.
- **tokio-tungstenite 0.29.x:** defaults are `connect` + `handshake` (no TLS).
  TLS features: `rustls-tls-native-roots`, `native-tls`.

## Decision

We will set `default-features = false` on both reqwest and tokio-tungstenite.
TLS is **not enabled by default**. Users opt in via feature flags:

```toml
[dependencies]
reqwest = { version = "0.13", default-features = false, features = ["json", "multipart", "form"] }
tokio-tungstenite = { version = "0.29", default-features = false, features = ["connect", "handshake"] }

[features]
default = []
rustls = ["reqwest/rustls", "tokio-tungstenite/rustls-tls-native-roots"]
native-tls = ["reqwest/native-tls", "tokio-tungstenite/native-tls"]
```

## Consequences

- Out of the box, the SDK connects over plain HTTP/WS only. This matches the
  primary use case (direct LAN access) and gives the smallest dependency tree
  and fastest compile times.
- Users behind a reverse proxy enable `rustls` or `native-tls` explicitly.
  This is a conscious choice, not a default they need to opt out of.
- rustls is the recommended TLS provider (pure Rust, no system dependency,
  cross-compilation friendly). native-tls is available for environments that
  require the platform's certificate store.
- The two TLS features are mutually exclusive in practice.
