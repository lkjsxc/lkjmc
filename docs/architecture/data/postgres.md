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

No migrations are implemented yet.
