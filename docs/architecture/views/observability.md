# Observability view

## Purpose

This view traces health, runtime observation, diagnostics, logs, and audit
signals.

## Status

implemented

## Signals

Runtime adapters return adapter-neutral observations. The daemon writes those
observations to PostgreSQL and exposes compact daemon status, doctor checks,
instance logs, bootstrap diagnostics, and audit records. Status distinguishes an
unconfigured database, connection failure, unavailable counts, and runtime lock
failure. Doctor sanitizes database URLs. Kubernetes observation includes pod
readiness, phase, restart count, and last error.

A missing external prerequisite is a diagnostic or guarded skip, never a healthy
observation. Live smoke is separate evidence from deterministic checks.

## Exact non-atomic boundaries

- Runtime observation and its PostgreSQL upsert are separate; a crashed daemon
  can leave the last persisted observation stale.
- `status` reads counts and runtime metadata independently, so it is a compact
  snapshot rather than a distributed atomic snapshot.
- Log reads are external runtime reads and are not committed with audit rows.

## Source trace

- `crates/lkjmc-daemon/src/commands/status_api.rs`
- `crates/lkjmc-daemon/src/commands/doctor_api.rs`
- `crates/lkjmc-daemon/src/support/instance_helpers.rs`
- `crates/lkjmc-daemon/src/runtime/kubernetes.rs`
- `crates/lkjmc-store/src/audit.rs`
