# PostgreSQL

## Purpose

This document defines how `lkjmc` uses PostgreSQL.

## Rules

- The installer provisions database role `lkjmc` and database `lkjmc`.
- Migrations live in `migrations/` with numeric names.
- All absolute timestamps use `timestamptz`.
- UUID primary IDs are used when records cross process boundaries.
- Constraints enforce uniqueness and referential integrity.
- Ledgers are append-only for money, commands, and audit events.
- Critical profile writes use leases or compare-and-swap revisions.

## Current status

Schema migrations are implemented and applied by the store migration helper and
installer. Current migrations create durable runtime, jar asset, player,
moderation, claims, and product tables described in [schema.md](schema.md).

## Bootstrap target

Playable bootstrap adds a generic asset registry, plugin installation records,
and bootstrap run ledgers. Until those migrations land, the existing jar asset
schema remains the current implementation boundary.
