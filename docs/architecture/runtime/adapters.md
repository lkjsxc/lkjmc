# Runtime adapters

## Purpose

This document defines the runtime adapter boundary that keeps durable instance
intent separate from process or cluster effects.


## Status

implemented

## Adapters

`local-process` starts rendered instance directories as local child process
groups, writes bounded logs, observes live process state, recovers healthy PIDs
after daemon restart, and stops through stdin, TERM, then KILL fallback.
`kubernetes` plans and applies owned cluster objects only after complete config
and real adapter checks are present.

## Boundary

PostgreSQL stores desired instance state. The daemon reconciler and command
handlers plan lifecycle work, then call the selected adapter for effects:
start, stop, observe, recover, logs, readiness, restart through stop/start, and
delete guardrails. Create handlers must resolve launch source, memory, port,
EULA acknowledgement, and template metadata before reporting success so every
created instance is startable or rejected with diagnostics. Adapter observations
are written back as observed state, health, PID when local, and safe diagnostics.

## Selection

JSON config uses `runtime.adapter`. Unknown values fail config parsing and must
not silently fall back. An adapter can be selectable only after it has real
effect execution, deterministic tests, status and doctor reporting, and opt-in
live smoke guidance when it depends on external infrastructure.

## Test-only seeded fault replay

Test-only adapters model a transaction, process effect, and observation write
without changing a pure planner or selecting a runtime adapter. A seeded
`ScenarioRunner` orders at most three instances, arms a `Failpoints` boundary,
and returns `Err` with a bounded transcript and effect state. Replaying a seed
must reproduce the same armed failure and transcript; a distinct seed may select
another order and transcript. This module compiles only under Rust test
configuration, so production invokes real adapters and cannot select a fault.

## Kubernetes adapter

A Kubernetes adapter observes cluster state and owns manifests, labels, service
discovery, storage, secret mounts, logs, readiness, stop, and delete semantics
before selection. Fake cluster state stored only in PostgreSQL is not a runtime
adapter. Default verification covers manifest planning; live cluster smoke is
opt-in.
