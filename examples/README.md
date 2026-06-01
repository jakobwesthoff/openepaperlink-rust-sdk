# Examples

Runnable programs demonstrating the SDK against a live OpenEPaperLink access
point. Each is built and run with:

```sh
cargo run --example <name> -- <args>
```

`<AP_URL>` is the access point's base URL, e.g. `http://192.168.1.100`. MACs are
the 16-hex-digit tag identifiers shown by `list_tags`.

## `list_tags`

Print a table of every tag the AP knows about — MAC, alias, battery, RSSI, and
content mode.

```sh
cargo run --example list_tags -- http://192.168.1.100
```

## `ws_monitor`

Connect to the AP's WebSocket and print every incoming event (tag updates,
system heartbeats, log lines) until interrupted.

```sh
cargo run --example ws_monitor -- http://192.168.1.100
```

## `get_config`

Dump the AP's system info (`/sysinfo`) and configuration (`/get_ap_config`) to
stdout.

```sh
cargo run --example get_config -- http://192.168.1.100
```

## `led_blink`

Flash a single tag's LED, or stop an ongoing flash.

```sh
# Start flashing
cargo run --example led_blink -- http://192.168.1.100 00007E231842B297

# Stop flashing
cargo run --example led_blink -- http://192.168.1.100 00007E231842B297 stop
```

## `upload_image`

Push a local JPEG to a specific tag. The AP converts and dithers it for the
target display.

```sh
cargo run --example upload_image -- http://192.168.1.100 00007E231842B297 picture.jpg
```

## `video_wall`

Span a single image across a grid of identical tags, forming one large picture.
The image is scaled to *cover* the combined canvas and dithered (Floyd–Steinberg)
against the panels' real color palette in one pass — so seams between tags line
up — then sliced into per-tag tiles and uploaded with the AP's own dithering
disabled.

The wall is described row by row with repeated `--row` flags, each a
comma-separated list of tag MACs. All tags must be the same hardware type.
`--padding` skips a bezel border around every tag so the image reads as
continuous across the physical gaps between panels. `--preview` additionally
writes a PNG of the composed result (tile boundaries outlined) for inspection.
`--dry-run` runs the whole pipeline but uploads nothing — pair it with
`--preview` to check the result before sending it to the tags.

```sh
# A 2×2 wall
cargo run --example video_wall -- http://192.168.1.100 mural.jpg \
  --row 00007E231842B297,00007E231842B298 \
  --row 00007E231842B299,00007E231842B29A

# A 1×3 strip with an 8px bezel and a preview image
cargo run --example video_wall -- http://192.168.1.100 panorama.png \
  --row 00007E231842B297,00007E231842B298,00007E231842B299 \
  --padding 8 --preview wall-preview.png

# Preview the result without uploading anything
cargo run --example video_wall -- http://192.168.1.100 panorama.png \
  --row 00007E231842B297,00007E231842B298,00007E231842B299 \
  --preview wall-preview.png --dry-run
```
