# Transport

## Purpose

This document defines daemon command transport contracts for CLI and browser
clients. Java daemon clients are withdrawn pending trusted identity/session
attestation.


## Status

implemented

## Listeners

The daemon starts one axum router on two listeners:

- a Unix domain socket for local CLI commands; and
- an optional TCP HTTP listener only when JSON `daemonHttp.enabled` is true.

The final effective JSON-and-CLI TCP address must parse as exactly
`127.0.0.1:PORT`; validation runs after all overrides. Hostnames, every other
`127/8` address, wildcard and unspecified addresses, IPv6 and IPv4-mapped IPv6
forms, and zero ports fail startup. Both listeners serve `POST /command` and compatibility `POST /` for
command envelopes. The Unix socket listener does not require bearer auth because
the socket path is local host state. TCP requires a constant-time bearer check.
Its root credential is limited to CLI-shaped operator requests. Java plugin and
proxy clients are not accepted while their adapters are withdrawn. Discord command
delegation is also withdrawn: no Discord transport subject or command surface is
accepted. Every registered command is `admin` or `operator`; unknown and
withdrawn requests deny instead of becoming open.

## HTTP contract

Requests contain the existing JSON command envelope: `requestId`, `actor`,
`command`, and `body`. Responses contain the existing command response shape.
One shared admission lease covers every supported `/`, `/command`, and `/web`
request before authentication, peer-denial audit, decoding, routing, or blocking
work. Synchronous database work stays below axum through the lease's blocking
worker; authentication, audit, command dispatch, and web rendering reuse that
one lease instead of spawning independent work. Transport admits eight leases
and keeps no application queue. A ninth request returns non-success
`command.queue_full` before any of those actions. The eight-second deadline
covers the whole response; expiry returns `command.deadline_exceeded` and never
claims that an effect completed or was externally cancelled. PostgreSQL checkout,
lock, and statement limits are shorter.

Request bodies are capped at 1 MiB. The outer HTTP timeout is 30 seconds.
Oversize bodies return HTTP 413, auth failures return HTTP 403 without echoing
token material, and unknown HTTP routes return a JSON 404. Invalid command JSON
is reported as a command error response without exposing request contents.

## Web routes

The same axum router serves `/web` paths. Browser sessions, CSRF checks, bearer
safe `/web/api/` mutations, and HTML rendering keep the path contract documented
in [../../web/routes.md](../../web/routes.md).

## Shutdown

SIGINT or SIGTERM first closes shared admission, then stops listener acceptance
through axum graceful shutdown, and waits for every lease-held worker to exit.
Already admitted auth, audit, web, local, and database work retains its permit
until exit. The daemon starts no command-driven child, network, filesystem,
plugin, proxy, or transfer effect during shutdown or recovery.

## Source owners

- Rust response shape: `lkjmc_core::command`.
- Daemon transport: `crates/lkjmc-daemon/src/transport/`.
- Web route adapter: `crates/lkjmc-daemon/src/web/routes.rs`.
- Java daemon clients: withdrawn; no Java source or plugin artifact owns one.
