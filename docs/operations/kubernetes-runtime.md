# Kubernetes runtime operations

## Purpose

This runbook owns cluster setup and opt-in smoke checks for the Kubernetes
runtime adapter.


## Status

implemented

## Required config

Operators must provide namespace, kubeconfig path or in-cluster mode, server
image reference, service type policy, storage class and size, readiness probe
settings, log limits, and CPU and memory requests.

## Safety checks

Use a dedicated namespace. The adapter only acts on objects with lkjmc ownership
labels for the target instance. Do not grant cluster-wide destructive
permissions to the daemon when namespace-scoped permissions are sufficient.

## Smoke

`LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` may create,
start, observe, fetch logs, stop, recover, and delete a test instance in the
configured namespace. Without the flag and namespace config, the script reports a
skipped live check.
