# Data architecture

## Purpose

This area owns PostgreSQL usage, schema shape, migration rules, and durable data
contracts.


## Status

implemented

## Table of contents

- [Claims](claims.md)
- [Economy](economy.md)
- [PostgreSQL](postgres.md)
- [Schema](schema.md)
- [Store](store.md)

## Current boundary

PostgreSQL is the only product store; SQLite is not product state. Store helpers own persistence effects, while pure cores own validation and transition planning. Transaction boundaries are enforced in executable Rust and PostgreSQL code and proved by focused crash, rollback, deadline, migration, and replay tests. The former regex/source inventory was deleted because it had drifted from executable behavior and classified symbol names rather than proving effects.

Profile, transfer, delivery, adventure, and runtime workflow rows record durable
intent and observation only. They never establish inventory receipt, player
arrival, or a runtime effect. Those transitions stay pending or failed until a
later trusted adapter supplies an authenticated, correlation-, revision-, and
fence-bound acknowledgement.

## Evidence and degraded behavior

Migrations and `lkjmc-store` are source evidence. If PostgreSQL is unavailable,
commands return the store failure and do not substitute plugin-local or
in-memory durable state. The monotonic change feed has an explicit retention and archive policy. Resume
returns a typed reload requirement below the active retained floor; archive
presence cannot hide a gap. The feed is not a broker or external-effect
authority.
