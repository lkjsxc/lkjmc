# Daemon

## Purpose

This area owns daemon transport, dispatch, health, and command catalog
contracts.


## Status

implemented

## Table of contents

- [Availability](availability.md)
- [Command lifecycle](command-lifecycle.md)
- [Command families](commands/README.md)
- [Observability](observability.md)
- [Status and doctor](status.md)
- [Transport](transport.md)

## Source tree

`crates/lkjmc-daemon/src/` keeps only these root files:

- `main.rs`: argument parsing, state construction, runtime startup.
- `app.rs`: shared daemon state, fixed startup config, pool, runtime, sessions.
- `authz.rs`: command authorization evidence and grant checks.
- `command_lifecycle.rs`: checked effect class, admission, and deadline policy.
- `dispatch.rs`: registry lookup, closed body-member validation, lifecycle gate, and response envelopes.

All other code lives under directories: `commands/`, `support/`, `runtime/`,
`reconcile/`, `assets/`, `templates/`, `transport/`, `web/`, and `tests/`.
`mod.rs` files declare modules and may re-export module entry points, but must
not contain handler logic.

## Implemented responsibilities

`lkjmc-daemon` serves axum HTTP command transport over the local Unix socket and
optional token-protected TCP listener. It loads JSON config, reads secret files
without printing them, uses PostgreSQL for durable state, exposes the command
registry catalog, and admits only the lifecycle classes documented in
[command lifecycle](command-lifecycle.md).

## Health and observability contracts

The status and doctor contract in [status.md](status.md) is implemented for
operator use. [Observability](observability.md) owns the shipped bounded local
event, metric, correlation, readiness, and support-diagnostic extension. Health
output stays aligned with the current-state ledger; no local event is an
independently attested observation.

## Truthfulness rule

Only locally proved observations and named PostgreSQL desired-state rows can
succeed. Every filesystem, network, process, plugin, proxy, transfer, observer,
and unproved database effect fails before its handler instead of falling through
to success.
