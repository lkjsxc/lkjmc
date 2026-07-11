# JVM research ideas

## Purpose

Imported platform-adapter candidates; none authorizes scheduler-thread effects.

## Catalog evidence

Source: supplied `experiments/catalog/jvm.md`. Each remains untested.

## Candidates

- `JV-ADAPTIVE-POLL` adaptive polling; `JV-LONG-POLL` revision waits.
- `JV-SSE` event stream; `JV-PUSH-REPAIR` push plus repair.
- `JV-FUTURE-PIPE` future/scheduler stages; `JV-VTHREAD` virtual-thread workers.
- `JV-DISPATCHER` typed effect dispatcher; `JV-SNAPSHOT-STORE` immutable snapshots.
- `JV-MENU-BUNDLE` validated bundle; `JV-MENU-SERVERMODEL` daemon view models.
- `JV-TRANSFER-ACK` durable acknowledgements; `JV-FOLIA-OWNERSHIP` ownership types.
- `JV-VELOCITY-ROUTE` owned topology reconciliation; `JV-TRANSFER-RESULT` outcomes.
- `JV-OFFLINE-UX` stable degraded paths; `JV-SHUTDOWN` task termination.
- `JV-PROTOCOL-HARNESS` real-client journeys.

## Decision boundary

Compare transport choices with meaningful execution models, then combine the
selected path with freshness, credential invalidation, transfers, and topology.
No candidate may block Minecraft scheduler threads.
