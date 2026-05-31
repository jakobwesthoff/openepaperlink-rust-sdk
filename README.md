# OpenEPaperLink SDK

A Rust library for talking to [OpenEPaperLink](https://github.com/OpenEPaperLink/OpenEPaperLink) access points. Push images to e-paper tags, subscribe to real-time events via WebSocket, fiddle with AP settings, and generally automate the things the web UI does — but from code, the way it should be.

Born out of wanting to programmatically poke at a bunch of electronic shelf labels without clicking through a browser. If you're running an OpenEPaperLink AP on your network and wish you had a proper API client for it, this is for you.

## What's in the box

- **[`docs/openepaper-ap-web-protocol.md`](docs/openepaper-ap-web-protocol.md)** — Reverse-engineered API manual covering every HTTP endpoint, WebSocket message type, the raw image format, and the JSON template language. Derived from source reading and validated against a live AP.
- **[`docs/adr/`](docs/adr/)** — Architectural decision records.

## License

[MIT](LICENSE)
