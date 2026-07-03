# Layout

## Purpose

This document defines the intended repository layout.

## Roots

- `docs/`: implementation contracts
- `contracts/`: machine-readable cross-language contract data
- `scripts/`: local entry points and checks, including verification tiers
- `migrations/`: PostgreSQL migrations
- `crates/`: Rust workspace crates: core, store, daemon, CLI, Discord, and xtask;
  daemon root files stay thin while commands, runtime, support, web, assets,
  reconcile, templates, transport, and tests live in subdirectories
- `platforms/jvm/`: Java common, Velocity, and Paper/Folia modules
- `config/`: locale catalogs and safe example defaults; user config remains JSON
- `tests/smoke/`: Java smoke harness sources used by guarded shell checks

## Generated paths

Jars, worlds, logs, database files, and `tmp/` are not committed.
