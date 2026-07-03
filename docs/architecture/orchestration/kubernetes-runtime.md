# Kubernetes runtime

## Purpose

This document defines Kubernetes orchestration behind the runtime adapter seam.


## Status

implemented

## Selection

`runtime.adapter` may be `kubernetes` only when the Kubernetes config block is
complete. Unknown or incomplete adapter config is a hard config error and must
not fall back to local processes.

## Ownership

Every object carries labels for network id, instance id, implementation,
template id, and managed-by marker. Instance delete and stop operations select
only objects with the exact ownership labels for that instance.

## Effects

- Start creates or scales owned workload, service, and storage references for
  the instance.
- Stop scales down or deletes the owned workload according to retention policy.
- Restart is stop then start while preserving configured durable storage.
- Observe reads typed pod readiness, phase, restart count, and last error.
- Logs read bounded container output through the daemon logs command shape.
- Recover rebuilds observation from existing owned objects after daemon restart.
- Delete removes only owned objects after the same guardrails as local runtime.

## Readiness

A backend is ready only when Kubernetes reports ready pods and the selected
Minecraft readiness check succeeds. Skipped cluster checks are reported as
skipped, never healthy.
