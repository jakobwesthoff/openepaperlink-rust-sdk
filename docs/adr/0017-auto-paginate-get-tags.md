# 17. Auto-paginate get_tags()

Date: 2026-05-31

## Status

Accepted

## Context

The AP's `GET /get_db` endpoint paginates responses at ~5000 bytes of
serialized JSON, using a `continu` key to indicate the next page offset.
Typical AP deployments have dozens to low hundreds of tags, spanning 1–5
pages.

We need to decide whether callers deal with pagination themselves, or the SDK
handles it internally.

## Decision

`get_tags()` will automatically follow pagination and return a complete
`Vec<TagRecord>`. A lower-level `get_tags_page(pos)` method will be
available for callers who need manual pagination control.

## Consequences

- The common case (`get_tags().await?`) just works — callers get all tags
  without knowing about pagination.
- All tags are collected in memory before returning. For typical deployments
  (< 500 tags), this is negligible.
- `get_tags_page(pos)` exposes the raw paginated response including the
  `continu` field for advanced use cases.
