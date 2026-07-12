# Control-plane research ideas

## Purpose

Imported daemon and orchestration candidates; none changes the current runtime.

## Catalog evidence

Source: supplied `experiments/catalog/control-plane.md`. Immutable source tip
`862f0e9db04330d23a339ec729098d322f7f1046` retains E-CONTROL's repaired
harness and rejected earlier Compose observations. Its source-only Compose
profile is intentionally absent from this evidence-only tree; the decision
selects no executor, journal, or actor. The other IDs remain untested.

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

E-CONTROL selects no executor, journal, or actor combination. Its journal
fault cannot prove exactly-once external effects after a post-launch crash, and
its config probe cannot observe runtime-memory application. It does not adopt a
daemon executor. Do not infer high availability from a lease experiment.
