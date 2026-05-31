# 21. TLS secure() compile-time gate

Date: 2026-05-31

## Status

Accepted

## Context

The `ClientBuilder` has a `secure()` method that switches the client to
HTTPS/WSS. However, TLS support is optional — it requires enabling the
`rustls` or `native-tls` feature flag (see ADR 0018). If a caller sets
`secure(true)` without a TLS feature enabled, the connection attempt fails
at runtime with an opaque error from reqwest or tungstenite.

## Decision

The `secure()` method on `ClientBuilder` is only available when a TLS
feature is enabled:

```rust
#[cfg(any(feature = "rustls", feature = "native-tls"))]
pub fn secure(mut self, secure: bool) -> Self { ... }
```

Without a TLS feature, calling `.secure()` is a compile error.

## Consequences

- Impossible to accidentally request TLS without a backend compiled in.
- The error is a clear compile-time message rather than an opaque runtime
  failure.
- Callers who need TLS must explicitly enable a feature flag in their
  `Cargo.toml`, which is the intended workflow.
