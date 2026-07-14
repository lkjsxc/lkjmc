# Domain to effect view

## Purpose

This view traces domain decisions to their effect-owning adapters.

## Status

implemented

## Mapping

| Domain decision | Durable owner | Effect owner |
| --- | --- | --- |
| Instance desired state | PostgreSQL instance records | daemon runtime adapter |
| Bootstrap plan | bootstrap run and steps | bootstrap effect adapter |
| Asset eligibility | jar and plugin asset records | download and install adapters |
| Player session and profile | player identity and session records | daemon/CLI lifecycle operations |
| Authorization | grants and audit records | daemon transport and web adapter |

Pure core planning supplies desired effects; it does not open sockets, launch
processes, write files, or call Kubernetes. Java plugins are local-safe
presentation only and submit no daemon commands pending trusted identity/session
attestation.

## Exact non-atomic boundaries

- `instance.start` crosses `start_runtime` and then updates desired state; a
  runtime start and its PostgreSQL update are separate operations.
- Each bootstrap effect except readiness is applied before its `bootstrap_steps`
  record is written. Readiness records a running probe before releasing its pool
  client, waits while the dedicated admission lock remains held, then reconnects
  to record its terminal result; neither ledger boundary is atomic.
- Asset bytes are downloaded or copied outside the database transaction that
  records asset metadata.

Recovery records an honest observation or failure; it does not claim an
unperformed effect succeeded.

## Source trace

- `crates/lkjmc-core/src/network_intent.rs`
- `crates/lkjmc-daemon/src/commands/bootstrap_api/apply/network_plan.rs`
- `crates/lkjmc-daemon/src/commands/bootstrap_api/apply.rs`
- `crates/lkjmc-daemon/src/commands/instance_lifecycle.rs`
- `crates/lkjmc-daemon/src/assets/plugin_install.rs`
- `crates/lkjmc-daemon/src/commands/player_session.rs`
