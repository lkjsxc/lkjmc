# Daemon

## Purpose

This area owns daemon transport, dispatch, health, and command catalog
contracts.


## Status

implemented

## Table of contents

- [Availability](availability.md)
- [Command families](commands/README.md)
- [Status and doctor](status.md)
- [Transport](transport.md)

## Source tree

`crates/lkjmc-daemon/src/` keeps only these root files:

- `main.rs`: argument parsing, state construction, runtime startup.
- `app.rs`: shared daemon state, config reload, pool, runtime, sessions.
- `authz.rs`: command authorization evidence and grant checks.
- `dispatch.rs`: registry lookup, dispatch entry, response envelopes.

All other code lives under directories: `commands/`, `support/`, `runtime/`,
`reconcile/`, `assets/`, `templates/`, `transport/`, `web/`, and `tests/`.
`mod.rs` files declare modules and may re-export module entry points, but must
not contain handler logic.

## Implemented responsibilities

`lkjmc-daemon` serves axum HTTP command transport over the local Unix socket and
optional token-protected TCP listener. It loads JSON config, reads secret files
without printing them, owns local process orchestration, uses PostgreSQL for
durable state, and exposes the command registry catalog.

## Health contract

The status and doctor contract in [status.md](status.md) is implemented for
operator use. Health output must stay aligned with the current-state ledger when
new PostgreSQL, runtime, bootstrap, or transport checks are added.

## Truthfulness rule

Command handlers must not claim success until PostgreSQL, filesystem, network,
probe, or process effects have completed. Unsupported effects fail explicitly
instead of falling through to success.
