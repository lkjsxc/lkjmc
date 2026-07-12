# Command effect boundaries

## Purpose

This contract records the source-backed effect boundary associated with each
registered daemon command without turning a handler response into effect proof.

## Values

A shard uses one of these values:

- `read`: returns derived or durable data with no intended mutation.
- `durable-write`: changes PostgreSQL-backed state.
- `runtime-effect`: reaches a runtime, process, filesystem, or download edge.
- `player-world-effect`: would require a trusted Minecraft adapter boundary.
- `mixed-effect`: combines durable and external work.
- `not-established`: the current handler source does not establish a uniform
  class at this contract boundary.

`idempotency` and `deadline` use `not-established` when no source-backed
replay result or deadline outcome exists. That is a compatibility fact, not a
permission to retry or a statement that a timeout succeeded.

## Current boundary

The selected shards make request member sets closed before dispatch. They do
not create a durable idempotency key, cancellation path, uniform timeout, or
exactly-once result. A synchronous response is not evidence of an external
runtime or player-world effect.

## Evidence

`contracts/commands/` records the per-command result. The real registrations
are `crates/lkjmc-daemon/src/commands/command_registrations.rs`, and
`scripts/check-contracts.py` proves only registry and source-data parity.
Future effect proof belongs to the responsible owner and must include its
failure and reconciliation evidence.
