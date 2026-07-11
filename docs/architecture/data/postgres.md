# PostgreSQL

## Purpose

This document defines how `lkjmc` uses PostgreSQL.


## Status

implemented

## Rules

- The installer provisions database role `lkjmc` and database `lkjmc`.
- Migrations live in `migrations/` with numeric names.
- The migration ledger stores a SHA-256 checksum. Apply and status verify every
  recorded name and checksum while holding a PostgreSQL advisory migration lock.
- Applied migration names are historical and are never renamed; for example,
  `006-ui-settings.sql` also creates economy and travel tables.
- All absolute timestamps use `timestamptz`.
- UUID primary IDs are used when records cross process boundaries.
- Constraints enforce uniqueness and referential integrity.
- Ledgers are append-only for money, commands, and audit events.
- Critical profile writes use leases or compare-and-swap revisions.

## Current status

Schema migrations are implemented and applied by the store migration helper and
installer. Current migrations create durable runtime, jar asset, player,
moderation, claims, and product tables described in [schema.md](schema.md).
Daemon runtime access goes through one PostgreSQL connection pool owned by
`AppState`; `database.poolSize` defaults to `8` and accepts `1..=64`. Single
direct connections are reserved for CLI migration flows, tests, and one-off
schema setup. Every acquired connection sets bounded PostgreSQL statement and
lock deadlines; migration serialization also uses that lock deadline. Pre-checksum
ledger rows are backfilled only while migration `038` is absent. Once that
upgrade migration is recorded, a NULL or mismatched checksum fails rather than
being silently rewritten.

## Bootstrap tables

The schema includes a generic asset registry, plugin installation records, and
bootstrap run ledgers. Existing jar asset tables remain available for the current
jar command surface while asset commands are implemented.
