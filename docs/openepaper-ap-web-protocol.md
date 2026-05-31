# OpenEPaperLink Access Point — Web API Manual

## Document provenance

This document was derived from **source code reading** of the
[OpenEPaperLink](https://github.com/OpenEPaperLink/OpenEPaperLink) repository
(the ESP32 AP firmware in `ESP32_AP-Flasher/` and its web frontend in
`ESP32_AP-Flasher/wwwroot/`), then **validated against a live AP**.

- **Repository:** `OpenEPaperLink/OpenEPaperLink`
- **Commit:** `5a4d646234e602ef4220464c4394c67811eaa11e` (2026-05-16)
- **Document created:** 2026-05-31
- **Validated against:** live AP at `192.168.11.186`, firmware 2.85, build env
  `ESP32_S3_16_8_YELLOW_AP`, build SHA `46c8b73fa0fd141e6a8a5652b1d855a2c1763924`.
  Endpoints verified via `curl`: `/sysinfo`, `/get_ap_config`, `/get_db`,
  `/check_file`, `/get_wifi_config`, `/get_ssid_list`, `/edit?list=`.
- **Tooling:** This document was extracted and phrased with the help of an AI
  agent (Claude). All technical claims were validated against source code and,
  where possible, against a live AP.
- **Cross-referenced with:** the OpenEPaperLink GitHub wiki (cloned
  2026-05-31). The wiki contains partial, usage-oriented documentation in
  `Led-control.md`, `Image-upload.md`, `Json-template.md`, `Content-cards.md`,
  `tagtype-notes.md`, and `Default-settings-for-new-Tags.md`. No comprehensive
  API reference exists in the wiki. Where the wiki provided additional detail
  (LED timing semantics, system variable names), it was validated against source
  before inclusion. Where the wiki was outdated (content mode 15 no longer
  exists, modes 27–29 are missing), this document follows the source.

### Conditional compilation

The AP firmware has multiple build variants. Features gated behind compile-time
flags are marked **[conditional]** throughout. The documented surface is the
**superset** of all builds. A specific AP may not expose all endpoints or fields.

Key compile-time gates:
- `HAS_EXT_FLASHER` — tag flasher, WebSocket `flashcmd`, TCP port 243
- `BOARD_HAS_PSRAM` — `sys.psfree` field in WebSocket system info
- `HAS_SUBGHZ` — sub-GHz channel fields in config and system info
- `HAS_H2` / `C6_OTA_FLASHING` / `HAS_TSLR` — radio module type flags
- `HAS_BLE_WRITER` — BLE writer capability flag
- `SAVE_SPACE` — reduced content mode set

---

## General conventions

### Base URL and transport

The AP runs an HTTP server on **port 80**. All paths in this document are
relative to `http://<ap-ip>/`. The WebSocket endpoint is at `ws://<ap-ip>/ws`
(or `wss://` if accessed via HTTPS proxy).

### Authentication

There is **no authentication** on any endpoint by default. All endpoints are
open. The `SPIFFSEditor` supports optional HTTP Basic Auth, but the AP
constructs it without credentials.

### CORS

All responses include:
```
Access-Control-Allow-Origin: *
Access-Control-Allow-Headers: content-type
```

### MAC address encoding

MAC addresses are encoded as uppercase hexadecimal strings with **reversed byte
order** (most-significant byte first in the string, least-significant byte at
index 0 of the internal `uint8_t[8]` array). Both 12-character (6-byte) and
16-character (8-byte) hex strings are accepted as input. The AP always emits
16-character uppercase strings in responses.

Example: internal bytes `[0x00, 0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F]`
→ string `"6F5E4D3C2B1A0000"`.

### Request encoding

- **POST** endpoints use `application/x-www-form-urlencoded` or
  `multipart/form-data` (for file uploads).
- **GET** endpoints use URL query parameters.
- One exception: `POST /save_wifi_config` accepts `application/json` body.

### Magic constants

These values have special meaning across the API and should be handled as
sentinel values, not regular data:

| Value | Context | Meaning |
|---|---|---|
| `3216153600` | `nextcheckin` | Tag is in deep sleep |
| `255` | `contentMode` | Tag was removed by a remote AP |
| `12` | `contentMode` | Content is managed by a different AP |
| `0` | `batteryMv` | No battery reading available |
| `1337` | `batteryMv` | Virtual/non-physical tag sentinel |
| `2600` | `batteryMv` | Capped reading, means "≥ 2.6V" |
| `100` | `RSSI` | Tag is the AP itself |
| `continu` | `get_db` response | Pagination key (intentionally misspelled) |

---

## 1. Tag management

These endpoints read and write tag state — querying the tag database,
configuring content modes, sending commands, and uploading content.

### 1.1 `GET /get_db` — List tags

Retrieves tag records from the AP's in-memory database. Used for initial load
and for fetching a single tag's details.

**Query parameters:**

| Parameter | Required | Description |
|---|---|---|
| `mac` | no | 12 or 16 char hex MAC address. Returns only that tag |
| `pos` | no | Pagination offset (default: 0) |

**Response:** `200 application/json`

```json
{
  "tags": [
    {
      "mac": "00007E23907FB299",
      "hash": "4eaaf64af5f3dcc50000000000000000",
      "lastseen": 1780232916,
      "nextupdate": 1780234085,
      "nextcheckin": 1780232976,
      "pending": 0,
      "alias": "",
      "contentMode": 4,
      "LQI": 124,
      "RSSI": -62,
      "temperature": 29,
      "batteryMv": 3062,
      "hwType": 51,
      "wakeupReason": 0,
      "capabilities": 225,
      "modecfgjson": "{\"location\":\"Berlin\",\"units\":\"0\"}",
      "isexternal": false,
      "apip": "0.0.0.0",
      "rotate": 0,
      "lut": 0,
      "invert": 0,
      "updatecount": 44,
      "updatelast": 1780228711,
      "ch": 11,
      "ver": 41
    }
  ],
  "continu": 25
}
```

**Pagination:** The AP chunks responses at approximately 5000 bytes of
serialized JSON. When more records exist, the `continu` field (note: not
`continue`) holds the offset for the next request. Absent when all records
are returned.

**Tag record fields:**

| Field | Type | Description |
|---|---|---|
| `mac` | string | 16-char uppercase hex, reversed byte order |
| `hash` | string | 32-char lowercase hex MD5 of current image data |
| `lastseen` | uint32 | Unix timestamp of last tag check-in |
| `nextupdate` | uint32 | Unix timestamp when AP will next generate content |
| `nextcheckin` | uint32 | Unix timestamp of expected next check-in. `3216153600` = deep sleep |
| `pending` | uint16 | Number of pending data transfers to this tag |
| `alias` | string | User-assigned display name (empty string if unset) |
| `contentMode` | uint8 | Content mode ID (see [content modes](#content-modes)) |
| `LQI` | uint8 | Link Quality Indicator (0–255) |
| `RSSI` | int8 | Received Signal Strength (dBm). `100` = AP itself |
| `temperature` | int8 | Tag-reported temperature in °C |
| `batteryMv` | uint16 | Battery voltage in millivolts (see magic constants for sentinels) |
| `hwType` | uint8 | Hardware type ID (maps to [tag type descriptors](#7-tag-type-descriptors)) |
| `wakeupReason` | uint8 | Why the tag last woke up (see [wakeup reasons](#wakeup-reasons)) |
| `capabilities` | uint8 | Capability bitmask (see [capabilities](#tag-capabilities)) |
| `modecfgjson` | string | JSON-encoded content-mode-specific configuration |
| `isexternal` | bool | `true` if managed by a different AP |
| `apip` | string | IP of the managing AP (relevant when `isexternal` is true) |
| `rotate` | uint8 | Display rotation: 0 = 0°, 1 = 90°, 2 = 180°, 3 = 270° |
| `lut` | uint8 | LUT mode: 0 = auto, 1 = full refresh, 2 = fast (no reds), 3 = fastest (ghosting) |
| `invert` | uint8 | Invert colors: 0 = normal, 1 = inverted |
| `updatecount` | uint32 | Total successful display updates |
| `updatelast` | uint32 | Unix timestamp of last successful update |
| `ch` | uint8 | Current radio channel |
| `ver` | uint16 | Tag firmware version |

---

### 1.2 `POST /save_cfg` — Configure tag display

Updates a tag's content mode, alias, and display settings. Only provided
parameters are changed.

**Request:** `application/x-www-form-urlencoded`

| Parameter | Required | Description |
|---|---|---|
| `mac` | yes | Tag MAC address |
| `contentmode` | no | Content mode ID |
| `alias` | no | Display name |
| `modecfgjson` | no | JSON string with mode-specific config |
| `rotate` | no | Rotation (0–3) |
| `lut` | no | LUT mode (0–3) |
| `invert` | no | Invert (0 or 1) |

**Response:** `200 text/plain`
- `"Ok, saved"` — success
- `"Error while saving: mac not found"` — unknown MAC

**Side effects:**
- Sets `nextupdate = 0`, triggering immediate content generation
- Broadcasts updated tag info to all WebSocket clients
- For content modes 5, 17, 18: saves current config as a backup, restores after sending

---

### 1.3 `POST /tag_cmd` — Send tag command

Sends a control command to a specific tag. Used for lifecycle management
(delete, reboot, sleep) and diagnostics (LED flash, channel scan).

**Request:** `application/x-www-form-urlencoded`

| Parameter | Required | Description |
|---|---|---|
| `mac` | yes | Tag MAC address |
| `cmd` | yes | Command string (see table) |

**Commands:**

| Command | Description |
|---|---|
| `del` | Delete this tag from the database |
| `purge` | Delete all inactive/timed-out tags from the database |
| `clear` | Clear pending data queue for this tag |
| `refresh` | Force content regeneration for this tag |
| `reboot` | Reboot the tag |
| `scan` | Tell the tag to scan for AP channels |
| `reset` | Reset tag settings to factory defaults |
| `deepsleep` | Put the tag into deep sleep |
| `ledflash` | Flash the tag LED with a default RGB pattern |
| `ledflash_long` | Flash the tag LED with a long red pattern (60 repeats) |
| `ledflash_stop` | Stop LED flashing |

**Response:** `200 "Ok, done"` or `400 "Error: mac not found"`

---

### 1.4 `GET /led_flash` — Direct LED control

Controls a tag's LED with a raw 12-byte pattern. Provides fine-grained control
over flash colors, timing, and sequencing beyond the preset patterns available
via `tag_cmd`.

**Query parameters:**

| Parameter | Required | Description |
|---|---|---|
| `mac` | yes | Tag MAC address |
| `pattern` | no | 24-character hex string encoding 12 bytes. Omit to send all zeros (stop) |

**LED flash pattern** (12 bytes packed, mapped directly from the hex string):

| Byte | Bits | Field | Description |
|---|---|---|---|
| 0 | 3:0 | `mode` | 0 = stop, 1 = flash (modes 2–15 reserved, currently treated as off) |
| 0 | 7:4 | `flashDuration` | Flash on-time in milliseconds. 0 = 0.5ms, 1–14 = that many ms, 15 = always-on (no off-time — destructive to battery). Values above 3 have diminishing visibility benefit; 2 is the recommended compromise |
| 1 | 7:0 | `color1` | RGB332 color for phase 1 |
| 2 | 3:0 | `flashCount1` | Number of flashes in phase 1 |
| 2 | 7:4 | `flashSpeed1` | Flash speed for phase 1 |
| 3 | 7:0 | `delay1` | Delay after phase 1 |
| 4 | 7:0 | `color2` | RGB332 color for phase 2 |
| 5 | 3:0 | `flashCount2` | Number of flashes in phase 2 |
| 5 | 7:4 | `flashSpeed2` | Flash speed for phase 2 |
| 6 | 7:0 | `delay2` | Delay after phase 2 |
| 7 | 7:0 | `color3` | RGB332 color for phase 3 |
| 8 | 3:0 | `flashCount3` | Number of flashes in phase 3 |
| 8 | 7:4 | `flashSpeed3` | Flash speed for phase 3 |
| 9 | 7:0 | `delay3` | Delay after phase 3 |
| 10 | 7:0 | `repeats` | Number of full cycle repeats |
| 11 | 7:0 | `spare` | Unused, set to 0 |

Colors use **RGB332** encoding: 3 bits red (bits 7–5), 3 bits green (bits 4–2),
2 bits blue (bits 1–0).

**Response:** `200 "ok, request transmitted"` or `400 "parameters are missing"`

---

### 1.5 `GET /getdata` — Retrieve raw image data

Returns the binary image data for a tag's current display content or a specific
pending queue item. Used for previewing what a tag is showing or about to show.

**Query parameters:**

| Parameter | Required | Description |
|---|---|---|
| `mac` | yes | Tag MAC address |
| `md5` | no | 16-char hex identifier of a pending queue item |

Without `md5`: returns the tag's current image. With `md5`: returns the
specified pending queue item.

**Response:** `200 application/octet-stream` (raw bytes) or `404 "File not found"`

**Note on image decoding:** The returned bytes may be compressed (zlib or G5)
depending on the tag firmware version and the tag type's `zlib_compression` /
`g5_compression` thresholds. Pixel format, bits-per-pixel, and color table are
defined by the [tag type descriptor](#7-tag-type-descriptors). See
[section 9](#9-raw-image-format) for the complete decoding pipeline.

---

### 1.6 `POST /imgupload` — Upload image for tag

Uploads a JPEG image to be rendered and sent to a tag. The AP converts it
to the tag's native display format.

**Request:** `multipart/form-data`

| Parameter | Required | Description |
|---|---|---|
| `mac` | yes | Tag MAC address |
| `file` | yes | JPEG image file |
| `dither` | no | Dithering: 0 = off, 1 = Floyd-Steinberg (default), 2 = ordered |
| `alias` | no | Update tag alias simultaneously |
| `rotate` | no | Rotation (0–3) |
| `lut` | no | LUT mode (0–3) |
| `invert` | no | Invert (0 or 1) |
| `contentmode` | no | Content mode (default: 24 = external image) |
| `ttl` | no | Time to live in minutes (0 = no expiry) |
| `preloadtype` | no | Preload image type ID (enables preload mode when present) |
| `preloadlut` | no | LUT for preloaded image (only with `preloadtype`) |

**Response:**
- `200 "Ok, saved"` — success
- `400 "mac not found"` — unknown MAC
- `409 "Come back later"` — AP is not in running state

**Side effects:** Stores image as `/temp/<mac>_<millis>.jpg`, sets content mode
and configuration, triggers immediate update.

---

### 1.7 `POST /jsonupload` — Upload JSON template for tag

Uploads a JSON template document to be rendered on a tag. The JSON template
format is defined by the AP's rendering engine (documented in the OpenEPaperLink
wiki, not in this protocol document).

**Request:** `application/x-www-form-urlencoded`

| Parameter | Required | Description |
|---|---|---|
| `mac` | yes | Tag MAC address |
| `json` | yes | JSON template content as a string |
| `ttl` | no | Refresh interval in minutes (default: 0) |

**Response:** `200 "Ok, saved"` or `400 "Missing parameters"` / `400 "mac not found in tagDB"`

**Side effects:** Stores JSON as `/current/<mac>.json`, sets `contentMode = 19`
(JSON template), triggers immediate update.

---

## 2. AP configuration

These endpoints read and write the access point's own settings — radio channel,
display brightness, timezone, and build-time capability flags.

### 2.1 `GET /get_ap_config` — Get AP configuration

Returns the merged configuration: compile-time capability flags plus runtime
settings from `/current/apconfig.json`.

**Response:** `200 application/json`

```json
{
  "C6": "1",
  "H2": "0",
  "TLSR": "0",
  "savespace": "0",
  "hasFlasher": "0",
  "hasBLE": "1",
  "hasSubGhz": "0",
  "apstate": "1",
  "channel": 0,
  "subghzchannel": 0,
  "alias": "",
  "led": 0,
  "tft": 20,
  "language": 2,
  "maxsleep": 0,
  "stopsleep": 1,
  "preview": 1,
  "nightlyreboot": 1,
  "lock": 0,
  "wifipower": 34,
  "timezone": "CET-1CEST-2,M3.5.0/02:00:00,M10.5.0/03:00:00",
  "sleeptime1": 0,
  "sleeptime2": 0,
  "ble": 0,
  "repo": "OpenEPaperLink/OpenEPaperLink",
  "env": "ESP32_S3_16_8_YELLOW_AP",
  "discovery": 0,
  "showtimestamp": 0
}
```

**Capability flags** (string `"0"` or `"1"`, determined at compile time):

| Field | Compile flag | Description |
|---|---|---|
| `C6` | `C6_OTA_FLASHING` | Has ESP32-C6 radio module |
| `H2` | `HAS_H2` | Has ESP32-H2 radio module |
| `TLSR` | `HAS_TSLR` | Has TLSR radio module |
| `savespace` | `SAVE_SPACE` | Reduced feature set build |
| `hasFlasher` | `HAS_EXT_FLASHER` | Has external tag flasher |
| `hasBLE` | `HAS_BLE_WRITER` | Has BLE writer |
| `hasSubGhz` | `HAS_SUBGHZ` | Has sub-GHz radio |

**Runtime configuration fields:**

| Field | Type | Default | Description |
|---|---|---|---|
| `apstate` | string | — | Current AP state (see [AP states](#ap-states)) |
| `channel` | uint8 | 0 | Radio channel (0 = auto) |
| `subghzchannel` | uint8 | 0 | Sub-GHz channel (0 = disabled) **[conditional]** |
| `alias` | string | `""` | AP display name (max 31 chars) |
| `led` | uint8 | 255 | LED brightness (0–255) |
| `tft` | uint8 | 255 | TFT display brightness (0–255) |
| `language` | uint8 | 0 | Language index |
| `maxsleep` | uint8 | 10 | Maximum tag sleep time in minutes |
| `stopsleep` | uint8 | 1 | Prevent sleep on tag update |
| `preview` | uint8 | 1 | Show image previews in web UI |
| `nightlyreboot` | uint8 | 1 | Enable nightly reboot at 03:56 |
| `lock` | uint8 | 0 | Lock configuration |
| `wifipower` | uint8 | 34 | WiFi TX power (ESP32 `wifi_power_t` enum) |
| `timezone` | string | `"CET-1CEST,M3.5.0,M10.5.0/3"` | POSIX timezone string |
| `sleeptime1` | uint8 | 0 | Night mode start hour (0–23) |
| `sleeptime2` | uint8 | 0 | Night mode end hour (0–23) |
| `ble` | uint8 | 0 | Enable BLE |
| `repo` | string | `"OpenEPaperLink/OpenEPaperLink"` | GitHub repo for OTA updates |
| `env` | string | *(build env)* | PlatformIO build environment name |
| `discovery` | uint8 | 0 | Enable discovery |
| `showtimestamp` | uint8 | 0 | Show timestamps |

**Side effect:** Triggers a UDP broadcast to discover other APs on the network.

---

### 2.2 `POST /save_apcfg` — Save AP configuration

Updates the AP's runtime configuration. Only provided parameters are changed.

**Request:** `application/x-www-form-urlencoded`

| Parameter | Description |
|---|---|
| `alias` | AP display name (max 31 chars) |
| `channel` | Radio channel |
| `subghzchannel` | Sub-GHz channel **[conditional]** |
| `led` | LED brightness (0–255) |
| `tft` | TFT brightness (0–255) |
| `language` | Language index |
| `maxsleep` | Max tag sleep minutes |
| `stopsleep` | Prevent sleep on update |
| `preview` | Show image previews |
| `nightlyreboot` | Enable nightly reboot |
| `lock` | Lock configuration |
| `wifipower` | WiFi TX power |
| `timezone` | POSIX timezone string |
| `sleeptime1` | Night mode start hour |
| `sleeptime2` | Night mode end hour |
| `ble` | Enable BLE |
| `discovery` | Enable discovery |
| `showtimestamp` | Show timestamps |
| `repo` | GitHub repository for OTA |
| `env` | Build environment name |

All parameters are optional. Only those present in the request are updated.

**Response:** `200 "Ok, saved"`

**Side effects:** Persists to `/current/apconfig.json`. Applies radio channel
and brightness changes immediately.

---

## 3. Variables (key-value store)

The AP maintains an in-memory key-value store that can be referenced in JSON
templates using `{key}` syntax. External systems (e.g., Home Assistant) can
set custom variables to feed data into templates.

The AP automatically maintains these system variables:

| Variable | Set in | Description | Example |
|---|---|---|---|
| `ap_ip` | `web.cpp` (every 5s) | AP's local IP address | `192.168.1.5` |
| `ap_ch` | `web.cpp` (every 5s) | AP radio channel (includes sub-GHz info if available) | `11` |
| `ap_tagcount` | `web.cpp` (every 60s) | Tag count summary | `42 / 45` or `42/45, 3 timeout` |
| `ap_date` | `web.cpp` (daily) | Current date (formatted per language setting) | `20-05-2023` |
| `ap_time` | `contentmanager.cpp` (on render) | Current time | `06:27:54` |

### 3.1 `POST /set_var` — Set a single variable

**Request:** `application/x-www-form-urlencoded`

| Parameter | Required | Description |
|---|---|---|
| `key` | yes | Variable name |
| `val` | yes | Variable value (string) |

**Response:** `200 "Ok, saved"` or `500 "param error"`

---

### 3.2 `POST /set_vars` — Set multiple variables

Sets multiple variables from a JSON object in a single request.

**Request:** `application/x-www-form-urlencoded`

| Parameter | Required | Description |
|---|---|---|
| `json` | yes | JSON object with key-value pairs |

Example body: `json={"temperature":"22.5","humidity":"65"}`

**Response:** `200 "JSON uploaded and processed"` or `400`

---

## 4. System and lifecycle

Endpoints for system information, time synchronization, database management,
rebooting, and WiFi configuration.

### 4.1 `GET /sysinfo` — Get system info

Returns build-time identity and hardware capabilities. This is a static
snapshot — it does not change at runtime (except `rollback` and `ap_version`).

**Response:** `200 application/json`

```json
{
  "alias": "",
  "env": "ESP32_S3_16_8_YELLOW_AP",
  "buildtime": "1768750871",
  "buildversion": "2.85",
  "sha": "46c8b73fa0fd141e6a8a5652b1d855a2c1763924",
  "psramsize": 8383863,
  "flashsize": 16777216,
  "rollback": true,
  "ap_version": 31,
  "hasC6": 1,
  "hasH2": 0,
  "hasTslr": 0,
  "hasFlasher": 0
}
```

| Field | Type | Description |
|---|---|---|
| `alias` | string | AP display name |
| `env` | string | PlatformIO build environment |
| `buildtime` | string | Build timestamp (epoch seconds, as string) |
| `buildversion` | string | Firmware version string |
| `sha` | string | Git commit SHA |
| `psramsize` | uint32 | PSRAM size in bytes |
| `flashsize` | uint32 | Flash chip size in bytes |
| `rollback` | bool | Whether OTA rollback is available |
| `ap_version` | uint16 | Radio module firmware version |
| `hasC6` | uint8 | Has C6 module (0/1) |
| `hasH2` | uint8 | Has H2 module (0/1) |
| `hasTslr` | uint8 | Has TLSR module (0/1) |
| `hasFlasher` | uint8 | Has external flasher (0/1) |

**Note:** This endpoint returns **numeric** values for capability flags (0/1),
unlike `get_ap_config` which returns **string** values ("0"/"1").

---

### 4.2 `POST /set_time` — Set system time

Synchronizes the AP's clock from an external source (e.g., Home Assistant)
without requiring internet access for NTP.

**Request:** `application/x-www-form-urlencoded`

| Parameter | Required | Description |
|---|---|---|
| `epoch` | yes | Unix timestamp in seconds |

**Response:**
- `200 "ok"` — time set successfully
- `400 "invalid epoch"` — value < 1600000000 (basic sanity check)
- `400 "missing 'epoch'"` — parameter absent

**Side effects:** Updates system clock, broadcasts system info via WebSocket.

---

### 4.3 `POST /reboot` — Reboot the AP

Saves the tag database, closes all WebSocket connections, and restarts the
ESP32.

**Response:** `200 "OK Reboot"` (connection will drop shortly after)

---

### 4.4 `GET /backup_db` — Download tag database

Forces a save of the in-memory tag database and returns the file as a download.

**Response:** `200` — attachment download of `/current/tagDB.json`

---

### 4.5 `POST /restore_db` — Restore tag database

Replaces the current tag database with an uploaded backup file.

**Request:** `multipart/form-data`

| Parameter | Required | Description |
|---|---|---|
| `file` | yes | Tag database JSON file (same format as `backup_db` output) |

**Response:** `200 "Ok, restored."`

**Side effects:** Destroys the current in-memory database entirely, then loads
the uploaded file as the new database.

---

## 5. WiFi configuration

These endpoints are used during initial AP setup or WiFi reconfiguration.

### 5.1 `GET /setup` — Serve WiFi setup page

Returns the WiFi configuration HTML page from the AP's filesystem.

**Response:** `200 text/html`

---

### 5.2 `GET /get_wifi_config` — Get WiFi configuration

Returns the stored WiFi credentials and network settings.

**Response:** `200 application/json`

```json
{
  "ssid": "MyNetwork",
  "pw": "password123",
  "ip": "",
  "mask": "",
  "gw": "",
  "dns": "",
  "mac": "34:CD:B0:0F:26:60"
}
```

| Field | Type | Description |
|---|---|---|
| `ssid` | string | WiFi network name |
| `pw` | string | WiFi password |
| `ip` | string | Static IP (empty = DHCP) |
| `mask` | string | Subnet mask (empty = DHCP) |
| `gw` | string | Gateway (empty = DHCP) |
| `dns` | string | DNS server (empty = DHCP) |
| `mac` | string | ESP32 WiFi MAC address (colon-separated, not tag MAC format) |

---

### 5.3 `GET /get_ssid_list` — Scan for WiFi networks

Returns nearby WiFi networks. Initiates an asynchronous scan if none is running
or the last scan is older than 30 seconds.

**Response:** `200 application/json`

```json
{
  "scanstatus": -2,
  "networks": []
}
```

| Field | Type | Description |
|---|---|---|
| `scanstatus` | int | Number of networks found, `-1` = scan in progress, `-2` = not started |
| `networks` | array | Up to 50 networks, each with `ssid`, `ch`, `rssi`, `enc` |

Clients should poll this endpoint until `scanstatus >= 0`.

---

### 5.4 `POST /save_wifi_config` — Save WiFi and reboot

Saves WiFi credentials and restarts the AP.

**Request:** `application/json` (exception — not form-encoded)

```json
{
  "ssid": "MyNetwork",
  "pw": "password123",
  "ip": "",
  "mask": "",
  "gw": "",
  "dns": ""
}
```

**Response:** `200 "Ok, saved"` (followed by reboot)

**Special case:** Setting `ssid` to `"factory"` performs a **factory reset** —
deletes all databases, configuration files, and firmware files, then deep-sleeps
and restarts.

---

## 6. OTA firmware updates

Endpoints for updating the ESP32 firmware, the radio module firmware, and
managing the LittleFS filesystem.

### 6.1 `GET /check_file` — Check file existence and integrity

Checks whether a file exists on LittleFS and returns its size and MD5 hash.
Used by OTA update flows to determine if files need downloading.

**Query parameters:**

| Parameter | Required | Description |
|---|---|---|
| `path` | yes | LittleFS file path (e.g., `/www/version.txt`) |

**Response:** `200 application/json`

```json
{
  "filesize": 4,
  "md5": "2ea8a8293148a18822156bd6b5befd20"
}
```

Returns `"filesize": 0` and `"md5": ""` if the file does not exist.

---

### 6.2 `POST /update_ota` — Initiate ESP32 OTA update

Starts a background firmware download and flash process. The AP downloads the
binary from the provided URL, verifies its MD5, and flashes it.

**Request:** `application/x-www-form-urlencoded`

| Parameter | Required | Description |
|---|---|---|
| `url` | yes | URL to download the firmware binary |
| `md5` | yes | Expected MD5 hash |
| `size` | yes | Expected file size in bytes |

**Response:** `200 "In progress"` or `400 "Bad request"`

**Side effects:** Stops AP operation, saves database, spawns background task.
Progress is reported via WebSocket `console` messages. On success, sends
`"[reboot]"` via WebSocket.

---

### 6.3 `POST /update_c6` — Flash radio module firmware

Initiates OTA flashing of the C6/H2 radio module. **[conditional: `C6_OTA_FLASHING`]**

**Request:** `application/x-www-form-urlencoded`

| Parameter | Required | Description |
|---|---|---|
| `url` | yes | Base URL containing firmware files |

**Response:** `200` or `400`

**Side effects:** Stops AP service, flashes radio module, brings AP back online.
Progress reported via WebSocket `console` messages.

---

### 6.4 `POST /rollback` — Rollback ESP32 firmware

Reverts the ESP32 to the previous OTA firmware partition.

**Response:**
- `200 "Rollback successful"` — followed by WebSocket `"[reboot]"` signal
- `400 "Rollback failed"` or `400 "Rollback not allowed"`

---

### 6.5 `POST /update_actions` — Execute cleanup actions

Reads `/update_actions.json` from LittleFS and deletes the files listed in its
`deletefile` array. Used after filesystem updates to remove obsolete files.

**Response:** `200 "Clean up finished"` or `200 "No update actions needed"`

---

### 6.6 `POST /littlefs_put` — Upload file to filesystem

Uploads a file to a specified path on LittleFS.

**Request:** `multipart/form-data`

| Parameter | Required | Description |
|---|---|---|
| `path` | yes | Destination path on LittleFS |
| `file` | yes | File content |

**Response:** `200 "Ok, file written"` or `507 "Error. Disk full?"`

---

### 6.7 `/edit` — File manager

Multi-method endpoint for browsing, reading, uploading, and deleting files on
LittleFS. Handled by the `SPIFFSEditor`.

#### `GET /edit?list=<path>` — List directory

| Parameter | Required | Description |
|---|---|---|
| `list` | yes | Directory path (e.g., `%2F` for root) |
| `recursive` | no | If present, lists recursively (skips `/www`, `/tagtypes`, `/current`) |

**Response:** `200 application/json`

```json
[
  {"type": "dir", "name": "current"},
  {"type": "file", "name": "log.txt", "size": 891}
]
```

In recursive mode, `name` includes the full relative path.

#### `GET /edit?edit=<path>` — Read file

Returns file content with appropriate MIME type.

#### `GET /edit?download=<path>` — Download file

Returns file as a download attachment.

#### `POST /edit` — Upload file

Standard multipart upload. Filename from the `data` field sets the path.

#### `DELETE /edit` — Delete file

**Form parameter:** `path` (required) — file path to delete.

**Response:** `200 "DELETE: <path>"`

#### `PUT /edit` — Create empty file

**Form parameter:** `path` (required) — file path to create.

Creates the file with a null byte if it doesn't exist. Returns `200` if it
already exists.

---

### Static file serving

| URL path | LittleFS source | Cache-Control |
|---|---|---|
| `/current/*` | `/current/` | `max-age=604800` (7 days) |
| `/tagtypes/*` | `/tagtypes/` | `max-age=300` (5 minutes) |
| `/*` (fallback) | `/www/` | default (serves `index.html`) |

---

## 7. Tag type descriptors

Tag type JSON files describe the display hardware of each tag model. They are
stored on LittleFS at `/tagtypes/<HH>.json` (where `<HH>` is the uppercase
2-digit hex hardware type ID) and served via HTTP at the same path.

```json
{
  "version": 1,
  "name": "Solum M3 BWR 2.9\"",
  "width": 296,
  "height": 128,
  "rotatebuffer": 1,
  "bpp": 2,
  "colortable": {
    "white": [255, 255, 255],
    "black": [0, 0, 0],
    "red": [255, 0, 0]
  },
  "perceptual": {
    "white": [255, 255, 255],
    "black": [0, 0, 0],
    "red": [200, 30, 30]
  },
  "shortlut": 1,
  "zlib_compression": "0100",
  "g5_compression": "0200",
  "highlight_color": 2,
  "options": ["led"],
  "contentids": [22, 1, 2, 3, 4, 8, 7, 19, 10, 11, 21],
  "usetemplate": 0,
  "template": {}
}
```

| Field | Type | Description |
|---|---|---|
| `version` | uint | Schema version (used for cache invalidation) |
| `name` | string | Human-readable tag model name |
| `width` | uint | Display width in pixels |
| `height` | uint | Display height in pixels |
| `rotatebuffer` | uint | Buffer rotation: 0 = none, 1 = 90° CW, 2 = 180°, 3 = 270° CW |
| `bpp` | uint | Bits per pixel: 1, 2, 3, 4, or 16 |
| `colortable` | object | Named color entries as `[R, G, B]` arrays |
| `perceptual` | object | Perceptual color table (preferred for display rendering) |
| `shortlut` | uint | 0 = no fast-refresh support, 1 = supported |
| `zlib_compression` | string | Min firmware version (hex) for zlib. `"0"` = unsupported |
| `g5_compression` | string | Min firmware version (hex) for G5. `"0"` = unsupported |
| `highlight_color` | uint16 | Highlight color index |
| `options` | string[] | Feature flags (e.g., `"led"`) |
| `contentids` | uint[] | Content mode IDs this tag type supports |
| `usetemplate` | uint | If non-zero, inherit templates from this hardware type ID |
| `template` | object | Content-mode-specific rendering templates |

---

## 8. WebSocket protocol

The WebSocket connection provides real-time updates from the AP: tag status
changes, system health, log messages, and OTA progress.

### Connection

- **URL:** `ws://<ap-ip>/ws`
- **Library:** `ESPAsyncWebServer` `AsyncWebSocket`
- **Reconnection:** recommended 5-second delay after close

### 8.1 Server → Client messages

All messages are JSON objects. Each message contains exactly **one** top-level
key that identifies the message type.

#### `logMsg` — Log message

```json
{"logMsg": "00007E23907FB299 update sent successfully"}
```

General log output from the AP. Messages starting with a 16-character hex string
relate to that tag MAC.

---

#### `errMsg` — Error message

```json
{"errMsg": "REBOOTING"}
```

Error or critical status messages.

---

#### `sys` — System info heartbeat

Sent approximately every **5 seconds**. Provides runtime health metrics.

```json
{
  "sys": {
    "currtime": 1780232927,
    "heap": 245760,
    "recordcount": 3,
    "dbsize": 98304,
    "littlefsfree": 1048576,
    "psfree": 4194304,
    "apstate": 1,
    "runstate": 2,
    "rssi": -55,
    "wifistatus": 3,
    "wifissid": "MyNetwork",
    "uptime": 86400,
    "lowbattcount": 0,
    "timeoutcount": 0
  }
}
```

| Field | Type | Presence | Description |
|---|---|---|---|
| `currtime` | uint32 | always | Current Unix timestamp |
| `heap` | uint32 | always | Free heap memory in bytes |
| `recordcount` | uint32 | always | Tag count (value cached, refreshed every 30s) |
| `dbsize` | uint32 | always | Database memory usage in bytes |
| `littlefsfree` | uint64 | always | Free filesystem space (value cached, refreshed every 30s) |
| `psfree` | uint32 | **conditional** | Free PSRAM. Only on builds with `BOARD_HAS_PSRAM` |
| `apstate` | uint8 | always | AP state (see [AP states](#ap-states)) |
| `runstate` | uint8 | always | Run status (see [run states](#run-states)) |
| `rssi` | int32 | always | WiFi RSSI in dBm |
| `wifistatus` | uint8 | always | WiFi status (Arduino `wl_status_t` enum) |
| `wifissid` | string | always | Connected WiFi SSID |
| `uptime` | uint64 | always | System uptime in seconds |
| `lowbattcount` | uint32 | **optional** | Tags with low battery. **Only present ~once per 60s** — absent from most heartbeats |
| `timeoutcount` | uint32 | **optional** | Timed-out tags. **Only present ~once per 60s** — absent from most heartbeats |

**Important for deserializers:** `lowbattcount` and `timeoutcount` are absent
from approximately 11 out of every 12 heartbeat messages. They must be modeled
as optional fields (e.g., `Option<u32>` in Rust).

---

#### `tags` — Tag info update

```json
{
  "tags": [
    { /* same fields as GET /get_db tag records */ }
  ]
}
```

Broadcast whenever a tag's status changes (check-in, config change, etc.).
Uses the same record format as `GET /get_db`.

---

#### `apitem` — AP discovery

```json
{
  "apitem": {
    "ip": "192.168.1.100",
    "alias": "Remote AP",
    "count": 15,
    "channel": "11",
    "version": "001F"
  }
}
```

Sent when a remote AP is discovered via UDP broadcast.

| Field | Type | Description |
|---|---|---|
| `ip` | string | IP address of the discovered AP |
| `alias` | string | AP display name |
| `count` | uint8 | Number of tags on that AP |
| `channel` | string | Radio channel |
| `version` | string | Firmware version as 4-char hex |

---

#### `console` — Serial/progress output

```json
{"console": "Progress: 45% 46080 102400"}
{"console": "Flashing succeeded", "color": "green"}
```

Used for OTA progress, flasher output, and serial passthrough.

| Field | Type | Description |
|---|---|---|
| `console` | string | Output text |
| `color` | string | Optional CSS color (`"green"`, `"red"`, `"yellow"`, `"white"`, `"silver"`, `"clear"`) |

**Special console messages:**
- `"[reboot]"` — AP is ready to reboot (frontend shows reboot button)
- `"-"` with `"color": "clear"` — clear the console display
- Messages starting with `"\r"` — overwrite the last output line
- Messages starting with `"<"` — prepend to the last indented line

---

### 8.2 Client → Server messages

Client-to-server WebSocket messages are only processed on APs built with
`HAS_EXT_FLASHER`. All messages are JSON. **[conditional]**

#### `flashcmd` — Flasher command

```json
{"flashcmd": 1}
```

| Value | Name | Description |
|---|---|---|
| 1 | `WEBFLASH_ENABLE_AUTOFLASH` | Start automatic tag flashing |
| 2 | `WEBFLASH_ENABLE_USBFLASHER` | Switch to USB/CLI flasher mode |
| 3 | `WEBFLASH_FOCUS` | Bring flasher to foreground |
| 4 | `WEBFLASH_BLUR` | Send flasher to background |
| 5 | `WEBFLASH_POWER_ON` | Power on external flasher port |
| 6 | `WEBFLASH_POWER_OFF` | Power off external flasher port |

**Response:** `{"flashstatus": 1}` sent to the originating client.

**Note:** Builds with `HAS_EXT_FLASHER` also listen on **TCP port 243** for raw
serial passthrough to the flasher hardware (used by `OEPL-flasher.py`).

---

## 9. Raw image format

Images are served as raw bytes via `GET /getdata` and `GET /current/<mac>.raw`.
The format depends on the tag type descriptor (see [section 7](#7-tag-type-descriptors)).
This section documents the complete decoding pipeline as implemented in the AP's
web frontend (`main.js`, `g5decoder.js`).

### 9.1 Decoding pipeline

The decoding happens in this order:

1. **Fetch** raw bytes from `/getdata?mac=...` or `/current/<mac>.raw`
2. **Zlib decompression** (if applicable)
3. **G5 decompression** (if applicable)
4. **Pixel interpretation** according to `bpp` and `colortable`

### 9.2 Zlib compression

Zlib compression is used when the tag's firmware version (`ver` field) is ≥ the
tag type's `zlib_compression` threshold (interpreted as hex). If
`zlib_compression` is `"0"`, zlib is not supported for that tag type.

**Format:**
```
Bytes 0-3:   (skipped — purpose undocumented, likely a length/header)
Bytes 4..N:  zlib-compressed payload (standard deflate)
```

After inflating:
```
Byte 0:      headerSize (uint8) — number of header bytes to skip
Bytes 1..headerSize-1:  header (skipped)
Bytes headerSize..end:  decompressed image data
```

### 9.3 G5 compression

G5 compression is a 1-bpp image codec based on Group 4 (MMR/T.6) fax encoding.
It is used when the tag firmware version is ≥ the tag type's `g5_compression`
threshold (hex). If `g5_compression` is `"0"`, G5 is not supported.

**G5 data starts with a header:**
```
Byte 0:      headerSize (uint8) — total header size in bytes
Bytes 1-2:   width (uint16, little-endian)
Bytes 3-4:   height (uint16, little-endian)
Byte 5:      bpp mode — 0 or 1 = normal, 2 = height is doubled (two planes)
```

The header is validated: width and height must match the tag type dimensions
(either `width×height` or `height×width` for rotated buffers), and `bpp mode`
must be ≤ 3.

After the header, the remaining bytes are G5-encoded 1-bpp data. The decoder
produces a standard 1-bpp packed buffer (8 pixels per byte, MSB first, white =
1, black = 0). When `bpp mode == 2`, the height is doubled, producing two
stacked planes (black plane followed by color plane).

### 9.4 Pixel formats

After decompression (if any), the raw image data is in one of these formats
based on the tag type's `bpp` value:

#### 1-bpp or 2-bpp (packed bitplanes)

The most common format for e-paper tags.

**1-bpp (monochrome):** 8 pixels per byte, MSB first. Each bit indexes into
the `colortable`: `0` = first color (typically white), `1` = second color
(typically black).

**2-bpp (three-color, e.g., black/white/red):** Two consecutive bitplanes.
The first plane occupies `width × height / 8` bytes, immediately followed by
the second plane of the same size. For each pixel:

```
pixelValue = blackBit | (colorBit << 1)
```

Where `blackBit` comes from the first plane and `colorBit` from the second
plane. The resulting 2-bit value indexes into the `colortable`:
- `0` = white (bit=0 in both planes)
- `1` = black (bit=1 in first plane)
- `2` = red/yellow (bit=1 in second plane)
- `3` = fourth color (bit=1 in both planes)

Bits within each byte are packed MSB-first: bit 7 is the leftmost pixel.

#### 3-bpp and 4-bpp (packed multi-bit)

Pixels are packed sequentially into bytes, MSB-aligned. Each pixel uses exactly
`bpp` bits. The pixel value indexes directly into the `colortable`.

For example, with 4-bpp: byte `0xA3` contains two pixels: pixel 0 = `0xA` (10),
pixel 1 = `0x3` (3).

Decoding reads across byte boundaries:
```
pixelValue = (data[byteIndex] << 8 | data[byteIndex+1])
             >> (16 - bpp - startBit)
             & ((1 << bpp) - 1)
```

#### 16-bpp (RGB565 or RGB332)

Two sub-formats are distinguished by data length:

**RGB565** (when `data.length == width × height × 2`): 2 bytes per pixel,
big-endian. Decode to 8-bit RGB:
```
R = ((pixel >> 11) & 0x1F) << 3
G = ((pixel >> 5)  & 0x3F) << 2
B = (pixel & 0x1F) << 3
```

**RGB332** (when `data.length == width × height`): 1 byte per pixel. Decode to
8-bit RGB:
```
R = ((pixel >> 5) & 0x07) * 36.3  (approximately: << 5, * 1.13)
G = ((pixel >> 2) & 0x07) * 36.3
B = (pixel & 0x03) * 83.2         (approximately: << 6, * 1.3)
```

### 9.5 Buffer rotation

The tag type's `rotatebuffer` field indicates that the raw pixel data is stored
in a rotated orientation relative to the display's natural dimensions:

| Value | Rotation | Canvas dimensions |
|---|---|---|
| 0 | None | `width × height` |
| 1 | 90° CW | `height × width` (swapped) |
| 2 | 180° | `width × height` (rendered upside-down) |
| 3 | 270° CW | `height × width` (swapped) |

When `rotatebuffer` is odd (1 or 3), the pixel buffer's effective dimensions
are swapped: the raw data has `height` columns and `width` rows.

### 9.6 Color table

The `colortable` in the tag type descriptor maps pixel values to RGB colors.
It is an object with named entries (e.g., `"white"`, `"black"`, `"red"`), each
being an `[R, G, B]` array. The pixel value (0, 1, 2, ...) indexes entries in
insertion order.

The optional `perceptual` table provides visually-adjusted colors for on-screen
rendering (e.g., a more muted red that better matches the e-paper appearance).
When present, the frontend prefers `perceptual` over `colortable`.

---

## 10. JSON template language

Content mode 19 ("JSON template") lets the AP render images from a declarative
JSON document. Templates can be pushed via `POST /jsonupload`, pulled from a
URL, or loaded from a file on LittleFS. The template is a JSON array of
drawing commands, processed top-to-bottom.

### 10.1 Document structure

A template is a JSON array where each element is an object with exactly one key
identifying the drawing command:

```json
[
  {"box": [0, 0, 296, 128, 1]},
  {"text": [10, 15, "Hello World", "fonts/bahnschrift20", 2]},
  {"line": [0, 50, 296, 50, 1]},
  {"image": ["/icon.jpg", 250, 10]}
]
```

### 10.2 Drawing commands

All coordinates are in pixels from the top-left origin of the display.

#### `text` — Draw text

```json
{"text": [x, y, "content", "fontname", color]}
{"text": [x, y, "content", "fontname", color, alignment]}
{"text": [x, y, "content", "fontname", color, alignment, size]}
{"text": [x, y, "content", "fontname", color, alignment, size, background_color]}
```

| Index | Parameter | Type | Description |
|---|---|---|---|
| 0 | `x` | int | X position |
| 1 | `y` | int | Y position |
| 2 | `content` | string | Text to display (supports template variables) |
| 3 | `fontname` | string | Font path or name (see [fonts](#105-fonts)) |
| 4 | `color` | color | Text color (see [colors](#104-colors)) |
| 5 | `alignment` | int | 0 = left (default), 1 = center, 2 = right |
| 6 | `size` | int | Font size (for TrueType fonts only, 0 = default) |
| 7 | `background_color` | color | Background color behind text (default: white) |

#### `textbox` — Paragraph text with word wrap

Renders text within a bounding box, breaking on spaces, hyphens, and carriage
returns. Only works with `.vlw` bitmap fonts.

```json
{"textbox": [x, y, width, height, "content", "fontname"]}
{"textbox": [x, y, width, height, "content", "fontname", color]}
{"textbox": [x, y, width, height, "content", "fontname", color, line_height]}
{"textbox": [x, y, width, height, "content", "fontname", color, line_height, alignment]}
```

| Index | Parameter | Type | Description |
|---|---|---|---|
| 0 | `x` | int | X position |
| 1 | `y` | int | Y position |
| 2 | `width` | int | Box width |
| 3 | `height` | int | Box height |
| 4 | `content` | string | Text content |
| 5 | `fontname` | string | Font path (`.vlw` only) |
| 6 | `color` | color | Text color (default: black) |
| 7 | `line_height` | float | Line spacing multiplier (default: 1.0, e.g., 1.25 for wider spacing) |
| 8 | `alignment` | int | 0 = left, 1 = center, 2 = right |

#### `box` — Filled rectangle

```json
{"box": [x, y, width, height, color]}
{"box": [x, y, width, height, color, border_color, border_width]}
```

#### `rbox` — Rounded rectangle

```json
{"rbox": [x, y, width, height, corner_radius, color]}
{"rbox": [x, y, width, height, corner_radius, color, border_color, border_width]}
```

#### `line` — Line

```json
{"line": [x1, y1, x2, y2, color]}
```

#### `triangle` — Filled triangle

```json
{"triangle": [x1, y1, x2, y2, x3, y3, color]}
```

#### `circle` — Filled circle

```json
{"circle": [x, y, radius, color]}
{"circle": [x, y, radius, color, border_color, border_width]}
```

#### `image` — Embedded JPEG image

```json
{"image": ["filename", x, y]}
```

Places a JPEG image from LittleFS at the given position. The image is rendered
at its native size (no scaling). Must be RGB baseline JPEG (not grayscale, not
progressive). The filename should start with `/`; if omitted, it is prepended
automatically.

#### `rotate` — Rotate display

```json
{"rotate": 0}
```

Rotates the entire canvas. Values: 0 = 0°, 1 = 90°, 2 = 180°, 3 = 270°.
This command affects the buffer mid-render — all subsequent drawing commands
operate in the rotated coordinate space.

### 10.3 Template variables

Text content in templates supports variable substitution using `{name}` syntax.

#### System variables

System variables are maintained by the AP and are always available:

| Variable | Description | Example |
|---|---|---|
| `ap_ip` | AP IP address | `192.168.1.5` |
| `ap_ch` | AP radio channel | `11` |
| `ap_tagcount` | Tag count summary | `42 / 45` |
| `ap_date` | Current date (language-formatted) | `20-05-2023` |
| `ap_time` | Current time (set at render time) | `06:27:54` |

Custom variables set via `POST /set_var` and `POST /set_vars` are also
available by their key name: `{temperature}`, `{humidity}`, etc.

Unknown variables are replaced with `"-"`.

#### JSON path variables

When a JSON template has both a `filename` (the template) and a `url` (a data
source), the AP fetches the URL and makes its JSON content available via
dot-path syntax. JSON path variables start with a dot: `{.path.to.value}`.

Given this data from the URL:
```json
{
  "sensor": {
    "readings": [
      {"temp": 22.5},
      {"temp": 23.1}
    ]
  }
}
```

The path `{.sensor.readings.0.temp}` resolves to `"22.5"`. Array elements are
accessed by numeric index.

Float and double values are rounded to 2 decimal places.

#### Arithmetic on JSON variables

A single arithmetic operation can be applied between two JSON path variables:

```
{.a.value}*{.b.factor}
{.temp.celsius}+{.offset}
{.price.gross}-{.discount}
{.total}/{.count}
```

Only one operator per expression. Chaining (`{.a}*{.b}/{.c}`) does **not**
work. The result is formatted with no decimal places.

#### HTTP request details for URL templates

When fetching the data URL, the AP sends:
- `If-Modified-Since` header with the last fetch timestamp (supports 304 caching)
- `X-ESL-MAC` header with the tag's MAC address
- Follows redirects (`HTTPC_STRICT_FOLLOW_REDIRECTS`)
- 20-second timeout
- HTTP/1.0 (via `useHTTP10(true)`)

The JSON response is limited by available memory on the ESP32 (~1000 bytes for
the variable document, though this is a practical rather than hard-coded limit).

### 10.4 Colors

Colors can be specified as numeric indices, named strings, or hex RGB:

| Value | Name | Color |
|---|---|---|
| 0 | `"white"` | White |
| 1 | `"black"` | Black (default for empty/missing) |
| 2 | `"red"` | Red |
| 3 | `"yellow"` | Yellow (if tag supports it) |
| 4 | `"lightgray"` | Light gray (pattern dithered) |
| 5 | `"darkgray"` | Dark gray |
| 6 | `"pink"` | Pink (pattern dithered) |
| 7 | `"brown"` | Brown |
| 8 | `"green"` | Green |
| 9 | `"blue"` | Blue |
| 10 | `"orange"` | Orange |
| — | `"#rrggbb"` | Custom RGB hex color |

Colors 4 and 6 use pattern dithering and are not suitable for small fonts.
On 2- or 3-color e-paper displays, non-native colors are dithered to the
available palette.

### 10.5 Fonts

Three font types are supported:

**Bitmap fonts (`.vlw`):** Stored in `/fonts/` on LittleFS. Referenced by path
(e.g., `"fonts/bahnschrift20"`). Pre-rendered at a fixed size; the `size`
parameter is ignored.

**TrueType fonts (`.ttf`):** Stored in `/fonts/` on LittleFS. Referenced by
filename (e.g., `"Inkfree.ttf"`). The `size` parameter in the `text` command
controls rendering size. Font files must be small enough to fit in the
filesystem.

**Legacy font names:** For backward compatibility, some old font names are
mapped automatically:
- `"glasstown_nbp_tf"` → `"tahoma9.vlw"`
- `"7x14_tf"` → `"REFSAN12.vlw"`
- `"t0_14b_tf"` → `"calibrib16.vlw"`

---

## Appendix A: Enumerations

### AP states

| Value | Name | Description |
|---|---|---|
| 0 | `AP_STATE_OFFLINE` | Offline, initializing |
| 1 | `AP_STATE_ONLINE` | Online and operational |
| 2 | `AP_STATE_FLASHING` | Flashing radio module firmware |
| 3 | `AP_STATE_WAIT_RESET` | Waiting for reset |
| 4 | `AP_STATE_REQUIRED_POWER_CYCLE` | Requires reboot |
| 5 | `AP_STATE_FAILED` | Failed to initialize |
| 6 | `AP_STATE_COMING_ONLINE` | Coming online |
| 7 | `AP_STATE_NORADIO` | AP without radio module |

### Run states

| Value | Name | Description |
|---|---|---|
| 0 | `RUNSTATUS_STOP` | Stopped |
| 1 | `RUNSTATUS_PAUSE` | Paused |
| 2 | `RUNSTATUS_RUN` | Running |
| 3 | `RUNSTATUS_INIT` | Initializing |

### Wakeup reasons

| Value | Name | Description |
|---|---|---|
| `0x00` | `WAKEUP_REASON_TIMED` | Scheduled timer wakeup |
| `0x01` | `WAKEUP_REASON_BOOT` | Normal boot |
| `0x02` | `WAKEUP_REASON_GPIO` | GPIO wakeup |
| `0x03` | `WAKEUP_REASON_NFC` | NFC wakeup |
| `0x04` | `WAKEUP_REASON_BUTTON1` | Button 1 pressed |
| `0x05` | `WAKEUP_REASON_BUTTON2` | Button 2 pressed |
| `0x06` | `WAKEUP_REASON_BUTTON3` | Button 3 pressed |
| `0xE0` | `WAKEUP_REASON_FAILED_OTA_FW` | Failed OTA firmware update |
| `0xFC` | `WAKEUP_REASON_FIRSTBOOT` | First boot |
| `0xFD` | `WAKEUP_REASON_NETWORK_SCAN` | Network scanning |
| `0xFE` | `WAKEUP_REASON_WDT_RESET` | Watchdog timer reset |

### Tag capabilities

Bitmask in the `capabilities` field. Only bits 0–7 are exposed in JSON (the
field is `uint8`); bit 8 (`CAPABILITY_IS_BLE = 0x100`) exists internally but
cannot appear in API responses.

| Bit | Value | Name | Description |
|---|---|---|---|
| 0 | `0x01` | `CAPABILITY_HAS_LED` | Tag has an LED |
| 1 | `0x02` | `CAPABILITY_SUPPORTS_COMPRESSION` | Supports compressed data |
| 2 | `0x04` | `CAPABILITY_SUPPORTS_CUSTOM_LUTS` | Supports custom LUTs |
| 3 | `0x08` | `CAPABILITY_ALT_LUT_SIZE` | Alternative LUT size |
| 4 | `0x10` | `CAPABILITY_HAS_EXT_POWER` | Has external power |
| 5 | `0x20` | `CAPABILITY_HAS_WAKE_BUTTON` | Has wake button |
| 6 | `0x40` | `CAPABILITY_HAS_NFC` | Has NFC chip |
| 7 | `0x80` | `CAPABILITY_NFC_WAKE` | Supports NFC wake |

### Content modes

Content modes define what the AP renders for a tag. Available modes per tag type
are listed in the tag type descriptor's `contentids` array. The full mode
definitions are in `content_cards.json` on the AP filesystem.

| ID | Name | Description |
|---|---|---|
| 0 | *(none)* | Not configured |
| 1 | Current date | Date display with optional sunrise/sunset |
| 2 | Count days | Day counter with red threshold |
| 3 | Count hours | Hour counter with red threshold |
| 4 | Current weather | Live weather via Open-Meteo |
| 5 | Firmware update | OTA firmware update for the tag *(temporary)* |
| 7 | Image URL | External JPEG image by URL |
| 8 | Weather forecast | 5-day forecast via Open-Meteo |
| 9 | RSS feed | RSS headlines |
| 10 | QR code | Full-screen QR code |
| 11 | Google calendar | Google Calendar via Apps Script |
| 12 | Remote content | Content from a different AP |
| 13 | Set segments | Segment display (debug) |
| 14 | Set NFC URL | Program NFC URL (requires `CAPABILITY_HAS_NFC`) |
| 16 | Buienradar | Dutch rain predictions |
| 17 | Send Command | Raw tag command *(temporary, dev only)* |
| 18 | Set Tag Config | Tag settings *(temporary)* |
| 19 | JSON template | Render from JSON template |
| 20 | Display a copy | Mirror another tag |
| 21 | AP info | Access point status display |
| 22 | Static image | JPEG from filesystem |
| 23 | Image preload | Preload for triggered display |
| 24 | External image | Image from `imgupload` or HA |
| 25 | Home Assistant | HA-provided image |
| 26 | Time Stamp | Button press timestamp tracker |
| 27 | Dayahead prices | Dynamic electricity tariffs |
| 28 | Set Tag Mac | Reprogram tag MAC |
| 29 | Current Time | Live clock |

Modes marked *(temporary)* save the previous configuration before sending and
restore it afterward.

---

## Appendix B: Scope exclusions

The following related topics are intentionally excluded:

- **UDP inter-AP synchronization** — internal AP-to-AP protocol, not exposed
  via the web API.
- **Tag radio protocol** — 802.15.4 communication between AP and tags.
