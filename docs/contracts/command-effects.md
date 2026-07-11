# Command effect boundaries

## Purpose

This contract distinguishes current daemon command envelope coverage from the
per-command effect, replay, and time boundaries still required for safe retries.

## Current scope

All 139 strict registry entries use generic-v&#49; request and response schemas.
They establish command names and a response envelope, not per-command bodies.
`CommandEnvelope` carries a caller request id, actor, command, and JSON body;
it has no idempotency key, effect class, deadline, or cancellation field.

Consequently, a request id is correlation only at this contract level. Some
handlers may have narrower database constraints or operation-specific retry
behavior, but no registry-wide exactly-once, replay, timeout, or cancellation
claim is made. Synchronous handler completion also does not establish a uniform
external-effect deadline.

## Domain effect classes

The following classes are the required classification vocabulary for future
per-command coverage; they are not yet registry fields.

- `read`: returns derived or durable data and has no intended mutation.
- `durable-write`: changes PostgreSQL-backed product state only.
- `runtime-effect`: starts, stops, transfers, downloads, or otherwise effects an
  external runtime or filesystem boundary.
- `player-world-effect`: bridges a confirmed daemon decision to a Minecraft
  inventory, teleport, or other adapter effect.
- `mixed-effect`: combines a durable write with one of the external classes.
- `reconcile`: observes or converges desired and observed runtime state.

A command can have one primary class plus documented subordinate effects. A
class is a planning and verification requirement, not evidence that an effect
succeeded.

## Required future boundary

Before a generic-v&#49; command claims complete schema coverage, its owner contract
must name its effect class, request and response schema, retry result, and
failure boundary. Mutating or effecting commands must also state one of:

- a durable idempotency key and replay result; or
- explicit at-most-once behavior with the caller retry prohibition.

Effecting commands must accept or derive a documented deadline and report the
state reached when that deadline expires. A timeout must not be reported as
success, and a late effect requires reconciliation evidence rather than a
second unguarded attempt. These are future contract requirements, not current
uniform daemon behavior.

## Evidence

Current envelope evidence is `crates/lkjmc-core/src/command.rs` and the two
shared schemas under `contracts/schemas/`. Registry evidence is
`contracts/commands.json`; `scripts/check-command-docs.py` checks registry
structure and schema paths but does not validate the classifications or execute
retries and deadlines.
