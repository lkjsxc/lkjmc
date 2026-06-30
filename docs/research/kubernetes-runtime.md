# Kubernetes runtime research

## Purpose

This document preserves research notes that informed the active Kubernetes
runtime adapter contract.

## Promoted contract

The executable contract now lives in
[Kubernetes orchestration](../architecture/orchestration/kubernetes-runtime.md),
[runtime adapters](../architecture/runtime/adapters.md), and
[Kubernetes operations](../operations/kubernetes-runtime.md).

## Safety notes

The adapter consumes the same desired instance state stored in PostgreSQL and
returns observed state, readiness, logs, and stop results through the runtime
adapter boundary. It defines object ownership, labels, service discovery,
storage classes, secret mounts, and log retention before cluster mutation.

## Verification notes

Deterministic manifest and adapter-boundary tests run by default. Actual cluster
smoke remains opt-in and guarded by an explicit environment flag and namespace.
