# OpenEPaperLink SDK

A Rust library for talking to [OpenEPaperLink](https://github.com/OpenEPaperLink/OpenEPaperLink) access points. Push images to e-paper tags, subscribe to real-time events via WebSocket, fiddle with AP settings, and generally automate the things the web UI does — but from code, the way it should be.

Born out of wanting to programmatically poke at a bunch of electronic shelf labels without clicking through a browser. If you're running an OpenEPaperLink AP on your network and wish you had a proper API client for it, this is for you.

## Quick start

```rust
use openepaperlink_sdk::{Client, StreamExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::builder("http://192.168.1.100").build()?;

    // List all known tags
    let tags = client.get_tags().await?;
    for tag in &tags {
        println!("{} — {}", tag.mac, tag.alias);
    }

    // Stream real-time events
    let mut stream = client.connect_ws().await?;
    while let Some(Ok(msg)) = stream.next().await {
        println!("{msg:?}");
    }

    Ok(())
}
```

`Client::builder` takes a base URL. Chain `.port(8080)` or `.timeout(Duration::from_secs(5))` before `.build()` if needed. Enable the `rustls` or `native-tls` feature for `https://` support.

## What's in the box

**Tags**
- `get_tags()` / `get_tag(mac)` — fetch the full tag list or a single record
- `save_tag_config(mac, cfg)` — update alias, content mode, rotate, LUT, etc.
- `tag_cmd(mac, cmd)` — send a `TagCommand` (e.g. force-refresh, reboot tag)
- `backup_db()` / `restore_db(data)` — dump and restore the AP's tag database

**Content**
- `upload_image(mac, bytes, opts)` — push a raw image to a tag
- `upload_json_template(mac, template)` — push a JSON template for server-side rendering

**LED**
- `led_flash(mac, pattern)` — blink a tag's LED with a `LedFlashPattern`
- `led_flash_stop(mac)` — stop an in-progress flash sequence

**AP config & system**
- `get_ap_config()` / `save_ap_config(cfg)` — read and write AP settings
- `get_sysinfo()` — firmware version, heap, flash size, build SHA, etc.
- `reboot()` / `set_time(epoch)` — system-level controls

**Variables**
- `set_var(key, value)` / `set_vars(map)` — set AP template variables

**WebSocket**
- `connect_ws()` → `EventStream` — typed stream of `WsMessage` variants: `SystemInfo`, `TagUpdate`, `ApItem`, `Log`, `Console`, `Error`

**Docs**
- [`docs/openepaper-ap-web-protocol.md`](docs/openepaper-ap-web-protocol.md) — reverse-engineered API manual covering every HTTP endpoint, WebSocket message type, the raw image format, and the JSON template language
- [`docs/adr/`](docs/adr/) — architectural decision records

## Examples

## Architecture

`Client` is defined in `src/client.rs`. Its methods are split across domain files (`tags.rs`, `config.rs`, `system.rs`, `content.rs`, `led.rs`, `variables.rs`, `ws.rs`) using separate `impl Client` blocks. Wire types live in `src/types/`. See the [ADRs](docs/adr/) for the rationale.

## Examples

Run any example with `cargo run --example <name> -- <AP_URL>`:

| Example | What it does |
|---|---|
| `list_tags` | Print a table of all tags with MAC, alias, battery, RSSI, and content mode |
| `ws_monitor` | Connect to the WebSocket and print every incoming event |
| `upload_image` | Push a local image file to a specific tag |
| `get_config` | Dump system info and AP config to stdout |
| `led_blink` | Flash a tag's LED (or stop an ongoing flash) |

## License

[MIT](LICENSE)
