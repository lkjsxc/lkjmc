# Layout

## Purpose

This document defines the intended repository layout.

## Roots

- `docs/`: implementation contracts
- `contracts/`: machine-readable cross-language contract data
- `scripts/`: local entry points and checks
- `migrations/`: PostgreSQL migrations
- `crates/`: Rust workspace crates: core, store, daemon, CLI, Discord, and xtask;
  daemon root files stay thin while commands, runtime, support, web, assets,
  reconcile, templates, transport, and tests live in subdirectories
- `platforms/jvm/`: Java common, Velocity, and Paper/Folia modules
- `config/`: locale catalogs and placeholder defaults; user config remains JSON
- `tests/`: placeholder smoke tree; executable smoke scripts live in `scripts/`

## Generated paths

Jars, worlds, logs, database files, and `tmp/` are not committed.
