# Lifecycle and recovery

## Purpose

This runbook defines the implemented runtime lifecycle and the limits of
recovery after a daemon restart.

## Status

implemented

## Lifecycle truth

`running` and `starting` are reconciled toward a running adapter instance.
`stopped` and `stopping` are reconciled toward absence. `restarting` performs a
stop then start and becomes `running` only after that sequence succeeds.
`suspended`, `deleting`, and `failed` are not restarted by reconciliation.
Autosuspend sets `suspended`; an operator start is the explicit wake action.

A runtime command is successful only after its adapter effect and required state
write complete. An unhealthy observation records a failure rather than proving a
process, pod, or player route is usable. Inspect `lkjmc instance list`,
`lkjmc doctor`, and bounded instance logs before declaring service restored.

## Daemon restart recovery

At daemon startup, recovery considers only stored rows marked healthy with a
valid PID. The local adapter adopts a still-existing process group or records it
absent. Kubernetes recovery observes labeled pods and records their observation;
it does not create, scale, or repair cluster objects. Rows without a qualifying
PID are not adopted by this pass.

Recovery is observation reconciliation, not rollback, backup restoration, or a
promise to restore player state. For database loss or corruption, use the
[backup and restore](backup-restore.md) drill. For an absent or unhealthy
workload, investigate the adapter diagnostic and deliberately start, stop, or
delete it as appropriate.

## Evidence boundary

Deterministic adapter and reconciler tests cover the lifecycle rules. The
Kubernetes smoke currently proves create, start, observe, logs, stop, and
delete; it does not execute a daemon-restart recovery scenario. A live recovery
claim requires an attempted guarded run with its cluster prerequisites and saved
output showing the observed workload state.
