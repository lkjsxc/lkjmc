# Control-plane research ideas

## Purpose

Imported daemon and orchestration candidates; none changes the current runtime.

## Catalog evidence

Source: supplied `experiments/catalog/control-plane.md`. Each remains untested.

## Candidates

- `CP-EXEC-ASYNC` async PostgreSQL; `CP-EXEC-BOUND` bounded sync workers.
- `CP-EXEC-MIXED` async plus effect workers; `CP-OPS-JOURNAL` durable operation IDs.
- `CP-IDEMPOTENCY` command classes; `CP-DEADLINE` client/operation cancellation.
- `CP-RUNTIME-KEYED` keyed guards; `CP-RUNTIME-ACTOR` fenced mailboxes.
- `CP-RUNTIME-DBLOCK` advisory-lock fencing; `CP-RECONCILE-EVENT` durable outcomes.
- `CP-NETWORK-SPEC` pure effects from intent; `CP-CAPABILITY` adapter capabilities.
- `CP-DOMAIN-SPLIT` durable-domain modules; `CP-PROCESS-SUPERVISE` isolated children.
- `CP-SHUTDOWN` in-flight-work policy; `CP-CONFIG-APPLY` real atomic reload.
- `CP-HA-LEASE` active reconciler lease.

## Decision boundary

Compare bounded, async, and mixed execution with keyed guards and actors. Add an
operation journal to two executors and combine the selected path with network
intent and workflows. Do not infer high availability from a lease experiment.
