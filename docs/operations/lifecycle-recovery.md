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
valid PID. The local adapter never adopts a stored PID: a still-existing group is
recorded unhealthy and fenced, while a missing group is recorded absent. Kubernetes
recovery observes labeled pods and records their observation; it does not create,
scale, or repair cluster objects. Rows without a qualifying PID are not examined
by this pass.

Recovery is observation reconciliation, not rollback, backup restoration, or a
promise to restore player state. For database loss or corruption, use the
[backup and restore](backup-restore.md) drill. For an absent or unhealthy
workload, investigate the adapter diagnostic and deliberately start, stop, or
delete it as appropriate.

## Evidence boundary

Deterministic process tests exercise the real local process-group fence and stop
boundaries. The durable local recovery test is skipped unless
`LKJMC_STORE_TEST_DATABASE_URL` names a disposable PostgreSQL database; when run,
it persists the fenced observation and reads it back. The same prerequisite gates
the cancellation-handler test, which verifies a fenced process leaves session,
temporary-instance, and refund rows unchanged. The guarded Kubernetes smoke
creates and starts an instance, then asserts only that its ID appears in `instance
list`; it ignores log-command failure and does not assert log content or post-stop/delete
state. It also does not execute daemon-restart recovery. A live recovery claim
requires a guarded run with saved output showing the observed workload state; see
[Kubernetes runtime](kubernetes-runtime.md).
