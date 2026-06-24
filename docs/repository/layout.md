# Layout

## Purpose

This document defines the intended repository layout.

## Roots

- `docs/`: implementation contracts
- `scripts/`: local entry points and checks
- `migrations/`: PostgreSQL migrations
- `crates/`: Rust workspace crates
- `platforms/jvm/`: Java common, Velocity, and Paper/Folia modules
- `config/`: default JSON config and locale catalogs
- `tests/`: smoke and integration assets

## Generated paths

Jars, worlds, logs, database files, and `tmp/` are not committed.
