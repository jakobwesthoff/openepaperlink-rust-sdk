# Project notes for coding agents

## Code structure

`Client` (defined in `src/client.rs`) has its methods split across domain
files via separate `impl Client` blocks. Adding a new endpoint means adding
to the appropriate domain file, not to `client.rs`.

| File | Domain |
|---|---|
| `src/tags.rs` | Tag CRUD, commands, backup/restore |
| `src/config.rs` | AP config read/write |
| `src/system.rs` | Sysinfo, reboot, set_time |
| `src/content.rs` | Image and JSON template uploads |
| `src/led.rs` | LED flash control |
| `src/variables.rs` | Key-value variable store |
| `src/ws.rs` | WebSocket event stream |

Wire types live in `src/types/` with one file per domain. Public re-exports
are explicit in `src/lib.rs` — no glob re-exports.

## Non-obvious conventions

- **MAC byte order is reversed.** The `Mac` newtype handles this. Wire hex
  strings are MSB-first, internal `[u8; 8]` is LSB-first. See `mac.rs`.
- **The AP returns HTTP 200 on some failures.** All write methods call
  `check_response_body()` which inspects the body text for error prefixes.
- **`reboot()` tolerates connection drops** (`is_connect` / `is_body`) since
  the AP restarts after responding.
- **Serde field names are inconsistent on the wire.** `TagRecord` mixes
  `lastseen` (all lowercase), `contentMode` (camelCase), `LQI` (uppercase).
  Each non-snake-case field has an explicit `#[serde(rename)]`.
- **`ApState` deserializes from both strings and integers** because
  `get_ap_config` returns `"1"` while the WebSocket heartbeat returns `1`.
- **`sys` heartbeat fields `lowbattcount`/`timeoutcount` are optional** —
  only present approximately once per 60 seconds, absent from most messages.
- **Query parameters are built manually** (no `reqwest/query` feature).
  `url_with_params()` does not percent-encode; all current callers pass only
  hex strings and numbers.

## Protocol reference

`docs/openepaper-ap-web-protocol.md` is the single source of truth for the
AP's HTTP and WebSocket API. It was reverse-engineered from the AP firmware
source and validated against a live AP.

## Testing

Unit tests cover pure logic only (Mac parsing, sentinel serde, WsMessage
dispatch, LED patterns, fixture-based TagRecord deserialization). No HTTP or
WebSocket mocking. Examples serve as integration tests against a live AP.
