# 19. Error policy for write endpoints

Date: 2026-05-31

## Status

Accepted

## Context

Several AP write endpoints return HTTP 200 even when the operation fails. For
example, `POST /save_cfg` returns `200 "Error while saving: mac not found"`
and `200 "Ok, saved"` through different code paths — both with the same status
code. The HTTP status alone cannot distinguish success from failure.

## Decision

All write methods inspect the response body text. If the body contains an
error indicator (does not start with "Ok" or similar success pattern), the
method returns `Error::Api { message }` with the full response text. On
success, the method returns `Ok(())` — the response body is discarded.

Read methods that return JSON do not need this treatment since deserialization
failure already surfaces errors.

## Consequences

- Callers get a clear `Result<(), Error>` from every write method — no need
  to inspect response strings themselves.
- The error detection is string-based, which is fragile if the AP changes its
  response wording. This is acceptable because the AP's error messages are
  simple and stable.
- The response body on success ("Ok, saved", "Ok, done") is discarded.
  Callers who need it can use the lower-level reqwest client directly.
