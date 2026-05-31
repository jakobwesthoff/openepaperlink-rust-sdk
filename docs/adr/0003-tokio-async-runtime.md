# 3. Use tokio as the async runtime

Date: 2026-05-31

## Status

Accepted

## Context

The library performs HTTP requests and maintains a persistent WebSocket
connection. Both are inherently asynchronous. We need to choose between
committing to a specific async runtime (tokio, async-std) or writing
runtime-agnostic code using abstraction traits.

## Decision

We will use tokio as the async runtime.

## Consequences

- Gives access to the strongest library ecosystem: reqwest, tokio-tungstenite,
  and the broader tokio utility crates.
- Callers must use a tokio runtime. This is the dominant choice in the Rust
  ecosystem and unlikely to be a practical constraint.
- Runtime-agnostic abstractions (e.g., trait-based executors, conditional
  compilation for async-std) are avoided, keeping the codebase simpler.
