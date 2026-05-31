# 16. Reboot-type endpoints treat connection drops as success

Date: 2026-05-31

## Status

Accepted

## Context

Several endpoints cause the AP to restart after responding:

- `POST /reboot` — sends `200 "OK Reboot"` then restarts
- `POST /save_wifi_config` with `ssid: "factory"` — may not complete the
  response before entering deep sleep
- `POST /update_ota` — the AP restarts after a successful flash

After the AP sends its response, the TCP connection drops. Depending on timing,
reqwest may surface this as a connection-reset error even though the command was
successfully accepted.

## Decision

Reboot-type methods return `Ok(())` if the AP accepted the command (200
response received). Connection resets that occur after the response was sent
are treated as expected behavior, not errors.

For `save_wifi_config("factory")` specifically, where the AP may drop the
connection before completing the response, a connection reset after the
request was fully sent is also treated as `Ok`.

## Consequences

- Callers get a clean `Ok(())` for the common case: command accepted, AP
  restarting.
- No spurious errors from post-response connection drops.
- Callers should expect the AP to be unreachable for several seconds after
  these methods return. Subsequent requests will fail until the AP completes
  its restart.
- The WebSocket stream (if connected) will end independently as described in
  ADR 0015.
