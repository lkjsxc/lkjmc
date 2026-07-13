# Kubernetes runtime

## Purpose

This document defines Kubernetes orchestration behind the runtime adapter seam.


## Status

implemented

## Selection

`runtime.adapter` may be `kubernetes` only when the Kubernetes config block is
complete. Unknown or incomplete adapter config is a hard config error and must
not fall back to local processes.

## Ownership

A mutable object must carry the exact managed-by and instance labels plus
operation-id and fence annotations. Before a mutation, the adapter must fetch
and compare those values with durable expected ownership, then bind the write to
the fetched UID or resourceVersion. Missing or mismatched metadata denies the
effect. The current adapter boundary cannot supply that durable expected
identity, and `kubectl` cannot precondition the multi-object delete atomically,
so stop and delete fail closed before mutation.

## Capability admission

Access checks and every command in an operation consume one monotonic total
deadline. Planning rejects storage, secret, configuration, logs, recovery, or
readiness requirements the adapter cannot prove. Missing client, namespace,
permission, budget, or capability is explicit `unsupported` or failure and does
not fall back to local process.

## Effects

- Start is unsupported before effect. Host-rendered work directories, jar paths,
  configuration, and token/secret paths are not mounted into the pod; `/data` is
  the only planned mount. Rendering a manifest is not a runnable launch claim.
- Stop and delete are unsupported before mutation until durable expected
  operation/fence ownership and race-safe preconditions reach the adapter.
- Observe reads typed pod readiness, phase, restart count, and last error within
  its bounded command budget; it is diagnostic, not launch proof.
- Logs remain bounded diagnostic access, not lifecycle capability.
- Recovery may observe but cannot mutate Kubernetes objects under this boundary.

## Readiness

A backend is ready only when Kubernetes reports ready pods and every configured
readiness condition is observed. Applying an object, seeing a pod phase, or
skipping a check is never readiness. Live Kubernetes remains guarded external
proof; deterministic capability-denial and plan tests always run.
