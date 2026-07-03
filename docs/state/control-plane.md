# Control plane state

## Purpose

This file records shipped Rust control-plane behavior.

## Status

implemented

## Core and store

- `lkjmc-core` owns typed IDs, command envelopes, the command registry, config
  defaults, validation, and deterministic planners.
- PostgreSQL migrations create durable tables for runtime, profiles, economy,
  achievements, shop, admin RBAC/audit, gameplay, moderation, claims, commands,
  Discord links, temporary adventures, transfers, and wake-and-join.
- `lkjmc-store` applies migrations and exposes typed helpers. Integration tests
  use per-test schemas and pass with parallel threads.
- The unused outbox module is removed; migration `034` drops the old table.

## Daemon and CLI

- `lkjmc-daemon` loads JSON config, owns a PostgreSQL pool, dispatches commands
  from the registry, and serves axum HTTP over TCP or Unix sockets.
- Command handlers remain synchronous; async work stays at the transport or
  Discord HTTP boundary.
- Mutating handlers write audit rows in the same transaction when multiple store
  writes are required.
- `lkjmc-cli` parses command families with unit coverage and sends command
  envelopes to daemon HTTP over a Unix socket.
- `instance.create.plan` reports structured diagnostics for missing jar assets,
  EULA, invalid ids, unsupported kinds, and other unstartable plans.
- `instance.list` includes connect address, proxy registration freshness,
  joinable state, and exact join-disabled reason when Velocity has reported
  backend registration.
- The daemon source root stays thin; domain logic lives under commands, runtime,
  support, assets, templates, reconcile, transport, web, and tests.
- Random teleport commands use profile-aware quotes, reservations, history, and
  per-profile cooldowns; the overworld profile costs zero points.

## Runtime boundaries

- PostgreSQL is the only durable product store.
- Generated secrets are never printed.
- Recovery reports are record-only audit-backed reports for operator review.
- Plugin and server downloads only use documented supported sources.
