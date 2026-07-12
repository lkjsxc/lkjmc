# Research adoption gate

## Purpose

Define the target boundary between research selection and a later architecture
change. It preserves the current implementation and does not register a product
mechanism.

## Status

planned

## Gate

A research result is only a proposal input. It becomes architecture work only
when an owner task first names its contract, inputs, effects, failure result,
authorization, durability, cleanup, deterministic proof, and guarded external
proof. The task must preserve PostgreSQL as sole durable truth and keep pure
Rust decisions separate from effect adapters.

The E-SYNTHESIS inputs are strict-contract generation, fail-closed credential
repair, deterministic verification, PostgreSQL cutover rehearsal, and the
already shipped local-only menu boundary. They are not source selections and
must not be copied from an experiment harness. The detailed evidence and
per-ID limits are in [E-SYNTHESIS](../research/decisions/e-synthesis-20260712.md).

## Hard boundaries

- Post-launch external-effect ambiguity blocks executor, journal, actor, lease,
  broker, and reconciliation adoption until a real idempotency/observation
  boundary is proved.
- Missing independent observer correlation blocks durable operation-history and
  observability adoption; a synthetic harness event cannot substitute for it.
- Java daemon adaptation remains withdrawn until trusted identity/session
  attestation, a real disposable adapter proof, and scheduler nonblocking proof
  exist. No scheduler callback may wait on database, filesystem, network, or a
  process.
- Kubernetes, remote, protocol-client, Bedrock, mobile, browser, Discord, and
  live-player observations remain guarded external evidence, not support claims.
- A later product change updates its owner contract before code and moves to
  [state](../state/README.md) only after source and deterministic proof exist.

## Non-goals

This gate does not alter commands, schema, config, adapters, provisioning,
external access, or controller state. It does not permit fake sync, process
management, downloads, event correlation, or successful results.
