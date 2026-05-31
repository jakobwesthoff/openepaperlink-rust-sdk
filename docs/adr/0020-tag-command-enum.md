# 20. TagCommand typed enum

Date: 2026-05-31

## Status

Accepted

## Context

The `POST /tag_cmd` endpoint accepts a `cmd` string parameter with values
like `"del"`, `"reboot"`, `"scan"`, etc. We need to decide whether the
library exposes this as a raw `&str` or a typed enum.

The library already invests in type safety for MAC addresses (`Mac` newtype),
content modes (`ContentMode` enum), battery readings (`Battery` enum), and
other wire values. Using a raw string for commands would be inconsistent.

## Decision

Tag commands use a `#[non_exhaustive]` `TagCommand` enum with 11 known
variants. Each variant maps to its wire string representation via a method
or custom serialization.

## Consequences

- Callers cannot pass an invalid command string — the compiler prevents it.
- The `#[non_exhaustive]` attribute allows adding new commands in minor
  releases as the AP firmware evolves.
- Consistent with the library's type-safe approach across the API surface.
