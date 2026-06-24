# Current state

## Purpose

This ledger states what is implemented now. If it conflicts with any target
contract, this file wins for current behavior.

## Implemented

- Repository documentation skeleton is implemented.
- Line-limit checks are implemented in `scripts/check-lines.py`.
- Documentation topology checks are implemented in `scripts/check-docs.py`.
- Cargo workspace scaffolding is implemented for five Rust crates.
- Gradle multiproject scaffolding is implemented for Java common, Velocity, and
  Paper/Folia modules.
- Dockerfile and Compose verify scaffolding are implemented.
- `scripts/verify.sh` runs docs, Rust, and Java foundation checks.
- `lkjmc-core` has pure Rust models for IDs, instances, jars, players,
  commands, audit events, and reconciliation effects.
- `lkjmc-core` parses and validates main and instance JSON config strings.
- PostgreSQL migrations create the current core schema foundation.
- `lkjmc-store` applies migrations and provides typed insert/read helpers for
  nodes, instances, jars, player profile records, commands, audit, and outbox.
- Store integration tests run against real PostgreSQL when the database URL
  environment variable is set.

## Not implemented

- Runtime daemon behavior is not implemented yet.
- CLI behavior is not implemented yet.
- Velocity plugin behavior is not implemented yet.
- Paper/Folia plugin behavior is not implemented yet.
- Installer is not implemented yet; `scripts/install.sh` exits with failure.
- Player synchronization runtime behavior is not implemented yet.
- Config loading from filesystem is not implemented yet.

## Verification status

The meaningful acceptance checks are foundation, pure-core, and store checks
until daemon and plugin behavior are added.
