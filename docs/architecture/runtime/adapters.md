# Runtime adapters

## Purpose

Define the boundary between JSON adapter selection and external runtime effects.

## Status

implemented

## Shared adapter boundary

`local-process` and `kubernetes` are immutable, shareable adapters. Effects take
`&self`; no daemon-wide adapter mutex exists. A keyed coordinator serializes
start, stop, observe, and reconcile for one instance. It holds the key-map lock
only while finding the instance guard, and never holds that map lock or a pooled
PostgreSQL connection while invoking an adapter. A blocked instance therefore
cannot block another instance.

Adapters declare capabilities before planning or applying work. Unsupported
readiness, storage, secrets, configuration, logs, or recovery fails explicitly.
Parsing adapter JSON is not a capability claim. Kubernetes additionally checks
`kubectl`, namespace access, and required resource verbs before mutation.

## Durable lifecycle boundary

A lifecycle operation first commits PostgreSQL intent with a monotonic
per-instance fence, operation id, and correlation id. It releases the connection,
rechecks ownership in a fresh transaction, then issues the effect. Observation
is committed only if the same operation still owns the current fence. A stale
or timed-out outcome cannot overwrite newer intent. Timeout records unknown or
failed outcome, never success.

Pure lifecycle planning maps durable intent plus observation and adapter
capabilities to `start`, `stop`, `observe`, or no-op. Reconciliation appends an
attempt and outcome for every pass. Repeating a satisfied operation is a no-op;
a pending crash window is recovered by observation before another effect.

## Shutdown

Shutdown closes admission before joining runtime work. New keyed work is
rejected, in-flight tasks are bounded by their deadlines, and local children are
stopped and reaped. A failed cleanup remains queryable and is never reported as
absence.

## Verification

`scripts/check-runtime-adoption.py` owns seven required probes. Its PostgreSQL
and process probes fail rather than skip. Kubernetes planning and capability
denial are deterministic; a guarded live cluster remains separate external
proof.
