# E-HC-CONTROL bounded high-cost control experiment

## Purpose

Test small, disposable control-plane alternatives without changing a product
schema, daemon, adapter, controller, command, or state claim.

## Catalog and baseline

This covers `HC-EVENT-ALL`, `HC-MULTI-DAEMON`, `HC-KUBE-OPERATOR`, and
`HC-MESSAGE-BROKER` from the [high-cost catalog](../ideas/high-cost.md). The
baseline is PostgreSQL durable truth with one daemon and direct PostgreSQL/HTTP
boundaries. The final decision records a disposition for every imported
high-cost ID, including IDs outside this slice.

## Hypothesis and slice

A private Compose PostgreSQL schema can show bounded event reconstruction and a
two-container, generation-fenced lease. A Compose Redis Stream can show the
additional bridge and consumer state required to mirror a PostgreSQL change
feed, alongside a PostgreSQL outbox pull. A disposable Kubernetes cluster may
allow a namespaced custom resource create/read/delete attempt, but needs
explicit cluster-scoped authorization for its temporary CRD.

The harness creates only the `lab` schema in a disposable database. Its event
stream has fixed instance IDs and a duplicate event ID; the projection is
rebuilt from the immutable rows. Two independently launched Compose client
containers acquire the same short lease in sequence; the old generation must
fail a guarded write after expiry while the new one succeeds. The broker probe
uses fixed Redis Stream IDs, reads them through a consumer group, acknowledges
them, and leaves an intentional pre-publish gap for the PostgreSQL pull to
find.

## Invariants and faults

- Eight source events and one duplicate yield eight immutable rows; rebuilding
  produces the same two stream projections.
- Daemon B cannot acquire A's live lease; after real expiry B gets generation
  two, A's generation-one write affects zero rows, and B's affects one.
- PostgreSQL pull claims all simple feed rows and observes the unmirrored gap.
  Redis receives, reads, and acknowledges every broker row, but cannot contain
  the deliberately unbridged row.
- The Kubernetes script always attempts local `kubectl` access. It proceeds
  only with `LKJMC_HC_KUBE_DISPOSABLE=1`, a namespace, and explicit
  cluster-scoped approval; otherwise it records `BLOCKED`, never a pass.

No probe treats a database event as an external effect, a lease as a fence for
an already-issued effect, Redis delivery as exactly once, or a custom resource
as a controller reconciliation result.

## Workload, writes, and rerun

The fixed seed is `20260712`; one Compose repeat uses eight events, twelve rows
per feed, and the two lease clients. Allowed tracked writes are
`docs/research/**`; raw logs are retained under an ignored output directory.
Docker, Compose, `postgres:16-alpine`, and `redis:7-alpine` are prerequisites.
Kubernetes also requires `kubectl`, reachable authorized credentials, a
throwaway namespace, and the explicit flags above.

```sh
ROOT="$(git rev-parse --show-toplevel)"
"$ROOT/docs/research/experiments/e-hc-control/run.sh" \
  --output /tmp/lkjmc-e-hc-control
```

The Compose and Kubernetes results are separate observations. Missing Docker,
Redis, PostgreSQL, or Kubernetes access is recorded as a failed or blocked
probe, not candidate support.
