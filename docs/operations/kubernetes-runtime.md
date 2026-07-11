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

Use a dedicated namespace. The adapter only acts on objects with lkjmc ownership
labels for the target instance. Do not grant cluster-wide destructive
permissions to the daemon when namespace-scoped permissions are sufficient.

## Smoke

`LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` builds a local
daemon and may create, start, observe, fetch logs, stop, and delete a test
instance in the configured namespace. It requires
`LKJMC_KUBERNETES_CONFIG`, `LKJMC_KUBERNETES_DATABASE_URL`, `kubectl`, cluster
credentials, and a disposable namespace. Without the guard it reports a skip;
with the guard but missing required values it fails rather than skipping.

The adapter can observe an existing labeled workload during daemon recovery, but
the current smoke does not execute that scenario. Do not report recovery as live
proved until a guarded recovery run records the observation.
