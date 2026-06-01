# Changelog

## Unreleased

### Added

- `get_tag_type` to fetch a tag type's display descriptor (dimensions and
  color palette) via `GET /tagtypes/<HH>.json`, plus `TagType` and `ColorEntry`
  types exposing width/height/bpp, the `colortable`/`perceptual` palettes, and
  supported `contentids`

## 0.9.0 — 2026-05-31

Initial release.

### Added

- `Client` with builder pattern accepting a base URL (`http://` or `https://`)
- Tag management: `get_tags`, `get_tag`, `tag_cmd`, `save_tag_config`,
  `backup_db`, `restore_db`
- Content uploads: `upload_image`, `upload_json_template`
- LED control: `led_flash`, `led_flash_stop` with `LedFlashPattern`
- AP configuration: `get_ap_config`, `save_ap_config`
- System: `get_sysinfo`, `reboot`, `set_time`
- Variables: `set_var`, `set_vars`
- WebSocket event stream via `connect_ws` returning a typed `EventStream`
  (`FusedStream` of `WsMessage` variants)
- `Mac` newtype handling the AP's reversed-byte-order hex encoding
- Semantic sentinel types: `Battery`, `NextCheckin`, `Rssi`
- Typed enums: `ContentMode`, `WakeupReason`, `ApState`, `RunState`,
  `TagCommand`, `Rotation`, `LutMode` — all `#[non_exhaustive]` with
  `Unknown` fallback variants
- Optional TLS via `rustls` and `native-tls` feature flags
- Comprehensive protocol documentation in `docs/openepaper-ap-web-protocol.md`
