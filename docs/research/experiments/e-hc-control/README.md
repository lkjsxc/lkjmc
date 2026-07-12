# E-HC-CONTROL disposable harness

## Purpose

Provide the real local/Compose harness for the bounded high-cost control
hypothesis. It is research-only and contains no product daemon, controller,
adapter, migration, command, or runtime registration.

## Table of contents

- [Source layout](#source-layout)
- [Safety and rerun](#safety-and-rerun)

## Source layout

- `compose.yml` starts disposable PostgreSQL, Redis, and two short-lived client
  containers.
- `lab.sql` owns the private schema for event, lease, fence, and feed probes.
- `daemon.sh` is the independent A/B lease client; it is not a daemon binary.
- `run.sh` coordinates the Compose workload and invokes the Kubernetes attempt.
- `kubernetes.sh` applies a temporary CRD and resource only after explicit,
  cluster-scoped authorization; `controltrial-crd.yaml` defines that resource.

## Safety and rerun

The Compose project is unique per invocation, uses only its own `lab` database,
and removes containers and volumes in an exit trap. Raw output must be an
ignored directory. The Kubernetes script first attempts client and server
access, then blocks unless the smoke, disposable-namespace, and cluster-scoped
flags are all explicit. It deletes only the CRD it created.

```sh
ROOT="$(git rev-parse --show-toplevel)"
"$ROOT/docs/research/experiments/e-hc-control/run.sh" \
  --output /tmp/lkjmc-e-hc-control
```

See the [hypothesis](../e-hc-control-20260712.md) for invariants and the
[run record](../../runs/e-hc-control-20260712.md) for actual observations.
