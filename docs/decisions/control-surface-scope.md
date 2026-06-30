# Control surface scope

## Purpose

This decision limits executable control-surface work to implemented, verified
surfaces.

## Decision

The current product scope is local-process orchestration through the daemon,
CLI, Velocity adapter, and Paper/Folia adapter. Web UI and Kubernetes runtime are
not active product targets for this repository state.

## Rationale

The repository must not carry future-facing goals that imply fake web routes,
fake cluster state, or unsupported runtime selection. Local-process behavior is
implemented and verified; web and Kubernetes work would require separate secure
surface design, external infrastructure, and live smoke prerequisites.

## Consequences

- Do not register web routes beyond the authenticated daemon/plugin HTTP command
  endpoint.
- Do not add `runtime.adapter` values for Kubernetes until a real cluster
  adapter, manifest planner, config validation, and opt-in live smoke exist.
- Research docs may preserve constraints for a future proposal, but they are
  guardrails, not a current implementation queue.
- A future change can reverse this decision only by updating this document,
  owner docs, current-state, and verification gates first.
