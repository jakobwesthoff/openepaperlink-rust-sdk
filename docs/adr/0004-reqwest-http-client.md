# 4. Use reqwest as the HTTP client

Date: 2026-05-31

## Status

Accepted

## Context

The AP's HTTP API requires form-encoded POST requests, multipart file uploads
(for image and firmware upload), and JSON response parsing. We need an HTTP
client that supports all of these on top of tokio.

## Decision

We will use reqwest.

## Consequences

- Built-in support for `application/x-www-form-urlencoded`, `multipart/form-data`,
  and JSON deserialization via serde.
- Pulls in a substantial dependency tree (hyper, http, tower, etc.), but this
  is acceptable for a library that already depends on tokio.
- The `reqwest::Client` can be reused across requests with connection pooling,
  which is beneficial for repeated polling of the AP.
