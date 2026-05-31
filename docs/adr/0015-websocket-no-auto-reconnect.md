# 15. WebSocket stream ends on disconnect, no auto-reconnect

Date: 2026-05-31

## Status

Accepted

## Context

The AP drops the WebSocket connection during reboots, OTA updates, and
network issues. The AP's own web frontend reconnects after a 5-second delay.
We need to define the SDK's behavior in this case.

Options considered:
- Stream ends, caller reconnects (simple, explicit)
- Auto-reconnect with backoff (convenient, hides state)
- Auto-reconnect with disconnect/reconnect events (most complete, most complex)

## Decision

The WebSocket stream ends on disconnect. The caller is responsible for
reconnecting by calling `client.connect_ws()` again.

The stream signals disconnection either by yielding `None` (stream exhausted)
or by yielding a final `Err` with the disconnect reason, depending on whether
the close was clean or unexpected.

## Consequences

- Simple, predictable behavior with no hidden reconnection logic.
- Callers implement their own retry strategy (fixed delay, exponential
  backoff, or none) based on their use case.
- No risk of silently reconnecting after a reboot and missing the fact that
  the AP state was reset.
- For callers who want auto-reconnect, a thin wrapper around the stream is
  straightforward to build on top.
