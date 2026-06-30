# Runtime and control surfaces

## Purpose

Keep non-local adapters and web controls truthful while the local runtime remains
the only implemented process adapter.

## Status

This is a guardrail, not an active executable blocker. Web UI and Kubernetes are
not current product targets under
[../../decisions/control-surface-scope.md](../../decisions/control-surface-scope.md).

## Runtime adapter seam

Document desired-state input, observed-state output, log ownership, readiness
probes, stop semantics, and unsupported operations for every adapter. A future
Kubernetes seam may define manifests and tests, but it must not register fake
cluster behavior.

## Web control surface

A web surface may be added only after the scope decision changes and only when
it calls the same daemon API as the CLI, binds privately by default,
authenticates requests, and audits every mutating action. It must not bypass
command handlers or invent separate state.

## Verification

Adapter units must run without a cluster. Any live cluster or browser smoke must
be opt-in and guarded by explicit environment flags.
