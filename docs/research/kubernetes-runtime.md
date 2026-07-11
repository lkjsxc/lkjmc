# Kubernetes runtime research

## Purpose

This concise history records research that informed the shipped Kubernetes
adapter; it is not the current behavior contract.

## Current owner evidence

The executable contract and current limits live in
[Kubernetes orchestration](../architecture/orchestration/kubernetes-runtime.md),
[runtime adapters](../architecture/runtime/adapters.md),
[Kubernetes operations](../operations/kubernetes-runtime.md), and
[control-plane state](../state/control-plane.md).

## Safety notes

The adapter consumes the same desired instance state stored in PostgreSQL and
returns observed state, readiness, logs, and stop results through the runtime
adapter boundary. It defines object ownership, labels, service discovery,
storage classes, secret mounts, and log retention before cluster mutation.

## Research boundary

Deterministic manifest and adapter-boundary tests are shipped evidence. Cluster
smoke is external proof: it requires `LKJMC_KUBERNETES_SMOKE=1`, `kubectl`, and
an authorized disposable namespace. A missing prerequisite is a skip, not a
claim that a candidate or cluster deployment passed.
