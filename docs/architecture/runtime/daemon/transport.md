# Transport

## Purpose

This document defines daemon command transport contracts for CLI, plugin, and
browser clients.

## Listeners

The daemon starts one axum router on two listeners:

- a Unix domain socket for local CLI commands; and
- an optional loopback TCP HTTP listener for JVM plugins and web operators.

Both listeners serve `POST /command` and compatibility `POST /` for command
envelopes. The Unix socket listener does not require bearer auth because the
socket path is local host state. The TCP command routes require
`Authorization: Bearer <token>` using constant-time credential comparison and the
current in-memory token so token rotation is honored without restart.

## HTTP contract

Requests contain the existing JSON command envelope: `requestId`, `actor`,
`command`, and `body`. Responses contain the existing command response shape.
Command dispatch stays synchronous below the transport boundary and is invoked
from axum through `spawn_blocking`.

Request bodies are capped at 1 MiB. Transport timeouts are 30 seconds. Oversize
bodies return HTTP 413, auth failures return HTTP 403 without echoing token
material, and unknown HTTP routes return a JSON 404. Invalid command JSON is
reported as a command error response without exposing request contents.

## Web routes

The same axum router serves `/web` paths. Browser sessions, CSRF checks, bearer
safe `/web/api/` mutations, and HTML rendering keep the path contract documented
in [../../web/routes.md](../../web/routes.md).

## Shutdown

SIGINT or SIGTERM triggers axum graceful shutdown for both listeners. Managed
child processes are left as durable runtime state and recovered by the daemon on
restart.

## Source owners

- Rust response shape: `lkjmc_core::command`.
- Daemon transport: `crates/lkjmc-daemon/src/transport/`.
- Web route adapter: `crates/lkjmc-daemon/src/web_routes.rs`.
- Java client: `platforms/jvm/common/src/main/java/com/lkjmc/common/daemon`.
