# Transport

## Purpose

This document defines daemon command transport contracts for CLI, browser, and the narrow plugin heartbeat client.


## Status

implemented

## Listeners

The daemon starts one axum router on two listeners:

- a Unix domain socket for local CLI commands; and
- an optional TCP HTTP listener only when JSON `daemonHttp.enabled` is true.

The final effective JSON-and-CLI TCP address must parse as exactly
`127.0.0.1:PORT`; validation runs after all overrides. Hostnames, every other
`127/8` address, wildcard and unspecified addresses, IPv6 and IPv4-mapped IPv6
forms, and zero ports fail startup. Both listeners serve `POST /command` and compatibility `POST /` for command envelopes. The Unix socket listener does not require bearer auth because the socket path is local host state. TCP authenticates database-backed, hashed bearer credentials. Generic command dispatch still rejects Paper and Velocity subjects.

TCP also serves `POST /plugin/v1/heartbeat`. Paper-compatible readiness heartbeats have an empty
body. A Velocity heartbeat carries one bounded JSON registration observation for every configured
backend; its authenticated instance identity still comes only from the credential. Success is HTTP
204 only after PostgreSQL commits the heartbeat and, for Velocity, the complete observed
registration set. The credential must have surface `paper` or `velocity`, principal kind
`instance`, principal ID equal to the managed instance, and sole scope
`lkjmc.instance.heartbeat`. The endpoint verifies that the credential surface matches the stored
instance kind. A plugin cannot name another heartbeat principal, submit player counts, invoke a
generic command, read sync state, or receive runtime authority. Missing, mixed-scope, expired,
wrong-kind, malformed, incomplete, duplicate, and oversized requests fail closed.

Velocity installation also fails unless its Rust-generated bounded backend inventory is present in
the live proxy registration set. The Velocity credential may belong to any persisted instance whose
kind is `velocity`; no ID is reserved. The proxy captures the expected ID/address set only after it
has observed each registration at startup. Every Velocity heartbeat then compares that snapshot to
the live proxy registry and reports exact registered, missing, or route-mismatch state. The daemon
requires exact set equality and independently compares every reported address with the persisted
typed fleet before storing it. Unsupported kinds, missing entries, unexpected IDs, unsafe hosts, or
invalid ports reject the transaction. Backend joinability requires a fresh backend heartbeat plus a
fresh registered Velocity observation in addition to daemon-owned process health. No database
desired-state row is promoted into an observed proxy effect.

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
claims that an external effect completed. After envelope decoding, deadline and
worker-failure responses preserve the validated client `requestId`; a synthetic
identifier is used only when decoding never yielded one. PostgreSQL checkout,
lock, and statement limits are shorter.

The request deadline is one monotonic instant captured with a successful
admission. It includes authentication, peer-denial audit, decoding, dispatch,
rendering, database connection, lock wait, statement execution, and response
selection. Each blocking action registers its `JoinHandle` before its caller can
suspend. Deadline reply or caller cancellation leaves that handle registered;
its worker-held lease remains until the work exits, and bounded cleanup joins the
handle without detaching work.
A result wins only before the instant. At or after it, the response is
`command.deadline_exceeded`. For admitted PostgreSQL desired-state mutations,
the worker remains responsible for recording a durable terminal outcome under
the same request ID even when the response deadline wins.

The pool gives every backend the eight-second request ceiling at connection
startup. A request worker calculates PostgreSQL checkout, lock, and statement
limits from its remaining monotonic budget, recalculates after checkout, and
sets the lock and statement limits before a handler query. A later operation
therefore cannot regain an earlier five-second allowance. The setup statement
inherits the prior bounded ceiling; the handler statement uses the remaining
budget. PostgreSQL cancellation ends a running statement; its worker remains tracked
until cancellation or normal completion has been joined, while its lease remains
through the running work.

Command request bodies are capped at 1 MiB; the heartbeat handler has a separate 32 KiB bound and
requires the body shape appropriate to the authenticated platform. The outer HTTP timeout is 30 seconds.
Oversize bodies return HTTP 413, auth failures return HTTP 403 without echoing
token material, and unknown HTTP routes return a JSON 404. Invalid command JSON
is reported as a command error response without exposing request contents.

## Web routes

The same axum router serves `/web` paths. Browser sessions, CSRF checks, bearer
safe `/web/api/` mutations, and HTML rendering keep the path contract documented
in [../../web/routes.md](../../web/routes.md).

## Shutdown

SIGINT or SIGTERM first closes shared admission, then stops listener acceptance
through axum graceful shutdown, and joins every registered blocking handle before
returning. Already admitted auth, audit, web, local, and database work retains
its lease until its worker exits; cancellation cannot release that lease early.
The daemon starts no command-driven child, network, filesystem, plugin, proxy,
or transfer effect during shutdown or recovery.

## Source owners

- Rust response shape: `lkjmc_core::command`.
- Daemon transport: `crates/lkjmc-daemon/src/transport/`.
- Web route adapter: `crates/lkjmc-daemon/src/web/routes.rs`.
- Empty-body heartbeat endpoint: `crates/lkjmc-daemon/src/transport/heartbeat.rs`.
- Bounded Java heartbeat reporter: `platforms/jvm/common/src/main/java/com/lkjmc/common/heartbeat/PluginHeartbeatReporter.java`.
