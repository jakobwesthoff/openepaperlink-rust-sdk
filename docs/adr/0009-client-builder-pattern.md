# 9. Client construction via builder pattern

Date: 2026-05-31

## Status

Accepted

## Context

The `Client` struct needs at minimum a host/IP to connect to the AP. Optional
configuration includes timeouts, a custom reqwest::Client, or a non-standard
port. We need a construction API that is consistent and extensible.

## Decision

We will use a builder pattern exclusively for constructing `Client`:

```rust
let client = Client::builder("192.168.1.100").build();
let client = Client::builder("192.168.1.100")
    .port(8080)
    .timeout(Duration::from_secs(10))
    .build();
```

There is no `Client::new()` shorthand.

## Consequences

- One consistent construction path. No ambiguity between `new()` and
  `builder()`.
- Easy to extend with new options without breaking the API (new builder
  methods are non-breaking additions).
- Slightly more verbose for the simplest case (`Client::builder(host).build()`
  vs. a hypothetical `Client::new(host)`), but the consistency is worth it.
- The builder can validate configuration at `.build()` time and return a
  `Result` if needed.
