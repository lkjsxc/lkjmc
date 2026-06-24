# Current state

## Purpose

This ledger states what is implemented now. If it conflicts with any target
contract, this file wins for current behavior.

## Implemented

- Repository documentation skeleton is implemented.
- Line-limit and documentation topology checks are implemented.
- Cargo workspace scaffolding is implemented for five Rust crates.
- Gradle multiproject scaffolding is implemented for Java common, Velocity, and
  Paper/Folia modules.
- Dockerfile and Compose verify scaffolding are implemented.
- `scripts/verify.sh` runs docs, Rust, Java, store, daemon/CLI, and process
  runtime checks.
- `lkjmc-core` has pure Rust models for IDs, instances, jars, players,
  commands, audit events, and reconciliation effects.
- `lkjmc-core` parses and validates main and instance JSON config strings.
- PostgreSQL migrations create the current core schema foundation.
- `lkjmc-store` applies migrations and provides typed insert/read helpers for
  nodes, instances, jars, player profile records, commands, audit, and outbox.
- `lkjmc-daemon` serves Unix socket JSON-RPC for `doctor`, `status`, and
  `audit.tail`.
- `lkjmc-daemon` has a token-protected loopback HTTP command endpoint.
- `lkjmc-daemon` can start, stop, restart, observe, delete, and tail logs for
  instances that have an explicit local launch command in their JSON config.
- The local runtime writes bounded process output under the configured log root
  and records observations in PostgreSQL when a database is configured.
- `lkjmc` CLI supports `doctor`, `status`, `config check`, `db migrate`,
  `db status`, `audit tail`, and the current instance list/create/start/stop,
  restart/delete/log commands.

## Not implemented

- Periodic desired-state reconciliation is not implemented yet.
- Instance file rendering, jar-backed launch, stdin/RCON graceful stop, and
  active player deletion guards are not implemented yet.
- Jar registry operations are not implemented yet.
- Velocity plugin behavior is not implemented yet.
- Paper/Folia plugin behavior is not implemented yet.
- Installer is not implemented yet; `scripts/install.sh` exits with failure.
- Player synchronization runtime behavior is not implemented yet.
- Config loading from filesystem is not implemented yet.

## Verification status

The meaningful acceptance checks are foundation, pure-core, store, daemon API,
and the command-driven process runtime slice. Process runtime checks require a
real PostgreSQL URL and are skipped by local verification when it is absent.
