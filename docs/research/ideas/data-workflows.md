# Data and workflow research ideas

## Purpose

Imported durable-consistency candidates; none changes database truth today.

## Catalog evidence

Source: supplied `experiments/catalog/data.md`. Each remains untested.

## Candidates

- `DW-TRANSFER-FLOW` fenced profile transfer; `DW-PROFILE-FORMAT` typed snapshots.
- `DW-DELIVERY-FLOW` durable delivery acknowledgement; `DW-ADVENTURE-FLOW` lifecycle.
- `DW-RUNTIME-FLOW` effect/observation history; `DW-REVISION` snapshot revisions.
- `DW-NOTIFY` invalidation-only notification; `DW-CHANGELOG` reconnect change feed.
- `DW-DELTA` revisioned deltas; `DW-FENCE` lease/effect fencing tokens.
- `DW-LEDGER-INVARIANT` verified balance entries; `DW-AUDIT-INTEGRITY` hash exports.
- `DW-RETENTION` archive/index policy; `DW-CANONICAL-SCHEMA` clean-schema cutover.
- `DW-SNAPSHOT-BACKUP` consistency markers; `DW-POOL-FAIRNESS` contention measure.
- `DW-CLOCK` deterministic planners.

## Decision boundary

Run selected workflows through crash and duplicate matrices. Compare snapshot
formats against complete documented fields; combine revisions with polling,
long-polling, push, and notification before adoption.
