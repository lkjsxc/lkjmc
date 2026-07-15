# Kubernetes runtime operations

## Purpose

This runbook owns cluster setup and opt-in smoke checks for the Kubernetes
runtime adapter.


## Status

implemented

## Required config

Operators may provide namespace, kubeconfig path or in-cluster mode, image,
service, storage, readiness, log, CPU, and memory settings for diagnostic
observation. These values do not enable launch. Host-rendered work directories,
jar paths, configuration, and token/secret paths have no truthful pod mount;
only `/data` is planned. Start therefore rejects before `kubectl`. Migrated
databases may persist Kubernetes diagnostic observed states.

## Safety checks

Use a dedicated namespace. One operation has one monotonic deadline shared by
access checks and commands. Stop and delete reject before mutation: the current
adapter boundary has no durable expected operation-id/fence to compare with
object annotations, and the multi-object `kubectl delete` path has no atomic UID
precondition. Missing or mismatched managed-by, instance, operation, fence, UID,
or resourceVersion metadata must deny any future destructive path. Do not grant
cluster-wide destructive permissions.

## Smoke

The retained guarded script may report external prerequisites, but this release
must not claim a Kubernetes lifecycle smoke: start, stop, and delete are
unsupported. Enabling the guard must fail before mutation rather than substitute
rendering, an absent client, or a skipped check for a live pass.

`scripts/check-runtime-adoption.py --probe adapter-capability-pass` always tests
local fail-closed launch, hung-command total deadline, and ownership denials. The
hung-command test runs by itself in an isolated harness process, records its
first failure output, and is never retried or overlapped with database stress.
Its deadline is not relaxed. It cannot skip because a live cluster is absent. A
guarded external skip is never
adapter proof.
