# Workflow and change propagation view

## Purpose

This view traces changes from durable intent through reconciliation and player
workflows.

## Status

implemented

## Propagation

Instance commands persist intent and invoke the selected runtime. The
reconciler periodically reads instances, presence, active sessions, and policy,
then observes, starts, or stops through the runtime adapter. Java heartbeat and
join/leave commands are withdrawn pending trusted identity/session attestation;
they do not update presence or sessions.

Bootstrap plans are computed from gathered facts. Apply executes ordered effects
and records each result. Transfer and temporary-instance workflows may persist
intent, but no Java adapter completes a player-facing move. The old temporary
transfer table and audit-only transfer command are absent.

## Exact non-atomic boundaries

- Presence or session writes and the next reconciler tick are asynchronous;
  there is no atomic change-propagation transaction.
- A bootstrap effect and the following step ledger write are separate calls.
- Durable transfer intent has no shipped Java consumer and is not proof a player
  arrived.

## Source trace

- `crates/lkjmc-daemon/src/reconcile/reconciler.rs`
- `crates/lkjmc-daemon/src/commands/instance_heartbeat.rs`
- `crates/lkjmc-daemon/src/commands/instance_wake_join.rs`
- `crates/lkjmc-daemon/src/commands/temporary_api/transfer.rs`
- `crates/lkjmc-store/src/data_workflows/transfer.rs`
