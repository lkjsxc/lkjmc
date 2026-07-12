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

No executor, journal, actor, lease, broker, operation history, or synthetic
observer event fills this gap. A later adapter proposal needs a real
idempotency/observation boundary, crash ordering, cancellation, cleanup, and
independent evidence before it can be admitted.

## Verification

`crates/lkjmc-core/src/kubernetes_tests.rs` proves only pure plan validation.
`scripts/check-command-lifecycle.py --probe effect-classes-enforced` proves
adapter commands are classified `denied-unproved`. Kubernetes live smoke is not
an available proof or support claim.
