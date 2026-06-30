# Control surface scope

## Purpose

This decision limits executable control-surface work to real, verified surfaces.

## Decision

The current scope promotes three daemon-backed control surfaces: local-process
orchestration, authenticated web control, and Kubernetes runtime orchestration.
A promoted surface is selectable only after its owner docs, config validation,
real effect adapter, deterministic tests, and guarded live smoke guidance exist.

## Rationale

The repository must not carry fake web routes, fake cluster state, or unsupported
runtime selection. Web control is acceptable only as a private authenticated
presentation layer over daemon commands. Kubernetes is acceptable only as a real
runtime adapter that owns cluster objects and reports real observations.

## Consequences

- Web routes must authenticate every request, delegate mutations to daemon
  command handlers, and audit safe mutation outcomes.
- `runtime.adapter = "kubernetes"` may parse only with complete Kubernetes
  config and a real adapter implementation.
- Research docs are historical guardrails; architecture and operations docs own
  active contracts for promoted work.
- Current-state moves a promoted surface to implemented only after source and
  verification evidence exist.
