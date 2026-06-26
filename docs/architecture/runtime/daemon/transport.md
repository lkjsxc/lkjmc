# Transport

## Purpose

This document defines daemon command transport contracts for CLI and plugin
clients.

## Unix socket JSON-RPC

The Unix socket server accepts one newline-terminated JSON command envelope per
connection and returns one JSON response envelope. CLI commands use this path by
default.

## HTTP command endpoint

The loopback HTTP endpoint accepts the same JSON envelope in the request body.
It is enabled unless the daemon starts with `--http none`. When enabled for
plugins, callers must send `Authorization: Bearer <token>`. The server reads the
full declared body before decoding JSON.

## Envelope

Requests contain `requestId`, `actor`, `command`, and `body`. Responses contain
the same request ID, `ok`, optional `body`, and optional structured `error`.
Error responses do not include secrets.

## Source owners

- Rust response shape: `lkjmc_core::command`.
- Daemon HTTP adapter: `crates/lkjmc-daemon/src/http_api.rs`.
- Java client: `platforms/jvm/common/src/main/java/com/lkjmc/common/daemon`.

## Java client

Java common uses Gson for request encoding and response decoding. Plugin
adapters consume `DaemonResponse.body()` as a typed JSON object through
`DaemonJson` helpers instead of parsing raw response strings.
