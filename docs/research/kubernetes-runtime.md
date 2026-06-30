# Kubernetes runtime

## Purpose

This document preserves Kubernetes runtime guardrails for a future proposal.
Kubernetes is not a current product target under
[control surface scope](../decisions/control-surface-scope.md).

## Adapter contract

If a future decision adds a Kubernetes adapter, it consumes the same desired
instance state stored in PostgreSQL and returns observed state, readiness, logs,
and stop results through the implemented runtime adapter boundary. It must
define object ownership, labels, service discovery,
storage classes, secret mounts, and log retention before any cluster mutation is
implemented.

## Safety rules

Cluster actions must not block Minecraft scheduler threads. Unsupported actions
return explicit daemon errors. Local-process behavior remains the only live
runtime unless a real cluster adapter, manifests, and verification gates are
implemented after the scope decision changes.

## Verification target

A future Kubernetes slice should add deterministic manifest/unit tests. An actual
cluster smoke is opt-in and guarded by an explicit environment flag.

## Current status

No Kubernetes adapter is implemented or registered, and `runtime.adapter` must
not accept Kubernetes until this decision changes.
