# Kubernetes runtime

## Purpose

This document defines the future Kubernetes runtime seam without registering a
fake cluster adapter.

## Adapter contract

A future adapter consumes the same desired instance state stored in PostgreSQL
and returns observed state, readiness, logs, and stop results through the runtime
adapter boundary. It must define object ownership, labels, service discovery,
storage classes, secret mounts, and log retention before any cluster mutation is
implemented.

## Safety rules

Cluster actions must not block Minecraft scheduler threads. Unsupported actions
return explicit daemon errors. Local-process behavior remains the only live
runtime until a real cluster adapter, manifests, and verification gates exist.

## Verification target

The first Kubernetes slice should add deterministic manifest/unit tests. An
actual cluster smoke is opt-in and guarded by an explicit environment flag.

## Current status

No Kubernetes adapter is implemented or registered.
