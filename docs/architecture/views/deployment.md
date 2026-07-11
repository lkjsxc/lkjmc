# Deployment view

## Purpose

This view traces deployable components, selected runtime adapters, and their
owned resources.

## Status

implemented

## Topology

The daemon is the control-plane process: it owns PostgreSQL access, Unix and
optional loopback HTTP listeners, and the selected runtime adapter. The
`local-process` adapter owns child JVM processes, instance directories, and
logs. The `kubernetes` adapter renders and applies owned workload, service, and
storage resources, then observes labeled objects. Velocity and Paper/Folia run
local-safe presentation only; they do not call the daemon pending trusted
identity/session attestation.

PostgreSQL is the durable product store. Runtime configuration selects exactly
one adapter; incomplete Kubernetes configuration is a hard error rather than a
local-process fallback.

## Exact non-atomic boundaries

- PostgreSQL desired state and a local child process or Kubernetes apply are
  separate systems; neither adapter effect is a database transaction.
- Kubernetes apply and later readiness observation are separate API calls.
- Plugin deployment to an instance directory and subsequent JVM plugin load are
  separate filesystem and JVM lifecycle steps.

## Source trace

- `crates/lkjmc-daemon/src/main.rs`
- `crates/lkjmc-daemon/src/runtime/local_adapter.rs`
- `crates/lkjmc-daemon/src/runtime/kubernetes.rs`
- `crates/lkjmc-core/src/kubernetes/observe.rs`
- `platforms/jvm/velocity/src/main/java/com/lkjmc/velocity/VelocityMotdAdapter.java`
