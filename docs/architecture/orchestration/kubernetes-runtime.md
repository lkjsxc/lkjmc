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

## Capability admission

Before an effect, the adapter proves that `kubectl` executes, the configured
namespace exists, and namespace-scoped authorization permits the exact resource
verbs required by the plan. Planning rejects storage, secret, configuration,
logs, recovery, or readiness requirements the adapter cannot prove. Missing
client, namespace, permission, or capability is explicit `unsupported` or
failure and does not fall back to local process.

## Effects

- Start renders deterministic namespace-scoped manifests from launch command,
  arguments, environment, server port, working directory, implementation kind,
  resource requests, and proved readiness/storage inputs. It applies only after
  capability and durable-fence ownership checks.
- Stop scales the exactly labelled workload to zero and observes zero replicas.
- Observe reads typed pod readiness, phase, restart count, and last error.
- Logs are supported only after log capability admission.
- Recovery observes exactly owned objects before deciding whether an effect is
  still needed.
- Delete removes only exactly owned objects after capability and player guards.

## Readiness

A backend is ready only when Kubernetes reports ready pods and every configured
readiness condition is observed. Applying an object, seeing a pod phase, or
skipping a check is never readiness. Live Kubernetes remains guarded external
proof; deterministic capability-denial and plan tests always run.
