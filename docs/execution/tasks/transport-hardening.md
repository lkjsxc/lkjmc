# Transport hardening

## Purpose

This task replaces fragile plugin-side daemon JSON parsing with typed helpers
while preserving asynchronous network behavior.

## Contract

- `DaemonRequest` remains an immutable Java record.
- `DaemonResponse` exposes a typed body instead of a raw JSON string map.
- HTTP request encoding and response decoding use a real JSON codec or a tested
  typed codec.
- Paper and Velocity adapters use helpers for strings, booleans, numbers,
  arrays, and objects.
- Scheduler-return code only touches Minecraft APIs on the appropriate scheduler
  bridge.

## Verification

Tests must cover request encoding, successful response decoding, daemon error
bodies, invalid JSON, HTTP failures, and at least one adapter/helper path that
no longer needs raw string extraction.
