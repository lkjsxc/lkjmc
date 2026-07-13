# Runtime adapters

## Purpose

Define the boundary between JSON adapter selection and external runtime effects.

## Status

partial

Missing: no adapter effect has a durable post-launch completion boundary.

## Current parsing

`local-process` and `kubernetes` remain accepted JSON adapter values. Parsing
and constructing an adapter are local configuration steps; neither is a process,
cluster, filesystem, network, readiness, log, recovery, stop, delete, or player
outcome claim. Unknown values still fail JSON parsing.

## Fail-closed boundary

The command lifecycle admits no adapter effect. All lifecycle, autosuspend,
recovery, temporary-cleanup, logs, readiness, and Kubernetes operations return
non-success before their registered command handlers run. Daemon startup does
not launch a reconciler or cleanup loop. PostgreSQL desired or observed rows do
not prove an external effect and cannot authorize one.

PostgreSQL now records correlation-, revision-, and fence-bound runtime effect
intent and observation state for later adapters. This data state machine is not
an executor, actor, broker, external lease, synthetic event, or proof of effect.
The A-DATA owner can create intent or fail it, but cannot mark an effect observed.
External commands remain `denied-unproved` and no reconciler is started.

A later adapter proposal needs a real authenticated idempotency/observation
boundary, crash ordering, cancellation, cleanup, and independent evidence before
it can advance the durable row or be admitted.

## Verification

`crates/lkjmc-core/src/kubernetes_tests.rs` proves only pure plan validation.
`scripts/check-command-lifecycle.py --probe effect-classes-enforced` proves
adapter commands are classified `denied-unproved`. Kubernetes live smoke is not
an available proof or support claim.
