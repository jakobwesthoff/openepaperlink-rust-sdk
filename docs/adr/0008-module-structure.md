# 8. Module structure with split impl blocks

Date: 2026-05-31

## Status

Accepted

## Context

The library exposes a `Client` struct with methods spanning multiple API
domains (tag management, AP configuration, system control, OTA, file
management, WiFi, variables, LED control). We need to organize the source
code so that related types and methods are grouped together without creating
import friction for callers.

Three patterns were considered:

1. **Feature-grouped modules** — types and `Client` methods co-located per
   domain file. Good discoverability but less idiomatic.
2. **Trait-based namespacing** — traits like `TagOperations`,
   `ConfigOperations` implemented on `Client`. Clean contracts but adds import
   friction (callers must import each trait to use its methods) and uses traits
   for namespacing rather than polymorphism.
3. **Split impl blocks** — types in a `types/` submodule, `Client` methods
   organized into domain files using separate `impl Client` blocks. Same file
   organization as traits, no import friction, idiomatic (used by octocrab and
   similar crates).

## Decision

We will use split `impl Client` blocks with types in a `types/` submodule.

The module layout:

```
src/
  lib.rs           — public re-exports
  client.rs        — Client struct definition, construction, shared helpers
  error.rs         — Error enum
  ws.rs            — WebSocket connection, WsMessage enum, stream types
  tags.rs          — impl Client: get_tags(), save_tag_config(), tag_cmd(), ...
  config.rs        — impl Client: get_ap_config(), save_ap_config()
  system.rs        — impl Client: get_sysinfo(), set_time(), reboot()
  variables.rs     — impl Client: set_var(), set_vars()
  files.rs         — impl Client: check_file(), upload_file(), list_files(), ...
  ota.rs           — impl Client: update_ota(), update_c6(), rollback()
  wifi.rs          — impl Client: get_wifi_config(), scan_ssids(), save_wifi_config()
  led.rs           — impl Client: led_flash()
  content.rs       — impl Client: upload_image(), upload_json_template()
  types/
    mod.rs         — re-exports
    tag.rs         — TagRecord, WakeupReason, Capability, TagDatabase
    config.rs      — ApConfig, ApCapabilities
    system.rs      — SystemInfo, SystemHeartbeat, ApState, RunState
    file.rs        — FileEntry, CheckFileResponse
    led.rs         — LedFlashPattern
    ap_item.rs     — ApListItem
    ws_message.rs  — WsMessage enum
    tag_type.rs    — TagTypeDescriptor, ColorTable
    content.rs     — ContentMode
    wifi.rs        — WifiConfig, SsidList, NetworkEntry
```

## Consequences

- Each domain file contains only the `impl Client` methods for that domain,
  keeping files focused and navigable.
- All wire types live under `types/`, making them easy to find and import
  independently of the client logic.
- Callers use `client.get_tags()` directly — no trait imports needed.
- Adding new API methods means adding to the appropriate domain file; no
  trait signatures to update.
