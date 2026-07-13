# Kubernetes runtime operations

## Purpose

This runbook owns cluster setup and opt-in smoke checks for the Kubernetes
runtime adapter.


## Status

implemented

## Required config

Operators must provide namespace, kubeconfig path or in-cluster mode, server
image reference, service type policy, storage class and size, readiness probe
settings, log limits, and CPU and memory requests. Launch manifests use the
instance command, args, env, configured server port, working directory, kind,
and readiness path; the adapter no longer substitutes a fixed port when config
specifies another value. Migrated databases accept Kubernetes observed states
such as `kubernetes-ready`, so status persistence must not fail only because
the runtime adapter is Kubernetes.

## Safety checks

Use a dedicated namespace. Startup capability admission requires `kubectl`, the
configured namespace, and namespace-scoped authorization for each planned verb.
The adapter only acts on objects with exact lkjmc ownership labels for the target
instance. Unproved storage, secret, config, log, recovery, or readiness support
is rejected before mutation. Do not grant cluster-wide destructive permissions.

## Smoke

`LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` requires
`LKJMC_KUBERNETES_CONFIG`, `LKJMC_KUBERNETES_DATABASE_URL`, `kubectl`, cluster
credentials, and an authorized disposable namespace. Without the guard it
reports the exact missing prerequisite as a skip; with the guard, any missing
value or capability fails. The smoke must record apply, readiness observation,
bounded logs, stop observation, recovery observation, and exact-label deletion
before claiming those live effects.

`scripts/check-runtime-adoption.py --probe adapter-capability-pass` always tests
deterministic plan and fail-closed capability behavior locally. It cannot skip
because a live cluster is absent. A guarded smoke skip is never adapter proof.
