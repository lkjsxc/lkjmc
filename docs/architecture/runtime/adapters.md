# Runtime adapters

## Purpose

This document defines the runtime adapter boundary that keeps durable instance
intent separate from process or cluster effects.

## Current adapter

`local-process` is the only selectable runtime adapter. It starts rendered
instance directories as local child process groups, writes bounded logs under the
configured log root, observes live process state, recovers healthy PIDs after a
daemon restart, and stops through stdin, TERM, then KILL fallback.

## Boundary

PostgreSQL stores desired instance state. The daemon reconciler and command
handlers plan lifecycle work, then call the selected adapter for effects:
start, stop, observe, recover, logs, readiness, restart through stop/start, and
delete guardrails. Adapter observations are written back as observed state,
health, PID when local, and safe diagnostics.

## Selection

JSON config uses `runtime.adapter`. Unknown values fail config parsing and must
not silently fall back. An adapter can be selectable only after it has real
effect execution, deterministic tests, status and doctor reporting, and opt-in
live smoke guidance when it depends on external infrastructure.

## Future adapters

A Kubernetes adapter must observe cluster state and own manifests, labels,
service discovery, storage, secret mounts, logs, readiness, stop, and delete
semantics before selection. Fake cluster state stored only in PostgreSQL is not
a runtime adapter.
