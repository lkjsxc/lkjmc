# Runtime and control surfaces

## Purpose

Keep promoted non-local adapters and web controls truthful while each surface
lands behind the daemon command and runtime seams.

## Status

This guardrail is closed for the promoted scope in
[../../decisions/control-surface-scope.md](../../decisions/control-surface-scope.md).
Local process orchestration, web control, and Kubernetes are promoted surfaces
with real behavior and guarded smokes. Future expansion must still avoid fake
product success.

## Runtime adapter seam

Document desired-state input, observed-state output, log ownership, readiness
probes, stop semantics, and unsupported operations for every adapter. The
Kubernetes seam owns manifests, real `kubectl` effects, typed observation,
recover, logs, stop, and delete with owned-label safety.

## Web control surface

The web surface calls the same daemon API as the CLI, binds privately by
default, authenticates requests, and audits mutating actions. Browser login,
session cookies, CSRF for form posts, and bearer-safe API paths are implemented
for the private operator surface.

## Verification

Adapter units must run without a cluster. Any live cluster or browser smoke must
be opt-in and guarded by explicit environment flags.
