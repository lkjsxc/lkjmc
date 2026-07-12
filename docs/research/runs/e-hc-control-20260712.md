# E-HC-CONTROL Compose run 2026-07-12

## Purpose

Record one disposable high-cost control comparison without adopting an event
store, replica daemon, broker, Kubernetes controller, schema, or product path.

## Identity and environment

- Experiment: [E-HC-CONTROL](../experiments/e-hc-control-20260712.md).
- Base: `4b9357a8e1a7949e0ebfe59c16af5196554f46cc`; hypothesis:
  `5a93208ce825a69fbb5639fdc29a54e22e5cf51d`; harness:
  `771b7f074fb0c6b1206619ba9189d0bb823d3b3d`.
- Host: Docker `29.5.3`, Compose `5.1.4`, `postgres:16-alpine`, and
  `redis:7-alpine`; fixed seed `20260712` and no production endpoint.
- Harness hashes: `run.sh`
  `40f45e6f5fee521f131f5f7ab4eb4a6291157edfd2f02589627ddf86eeb04631`;
  `lab.sql`
  `bd0487d8eef5b3aa77b5f0afe7a8734b04b9f94362a356b196cfecd4e5ec021f`;
  `kubernetes.sh`
  `450d5943d492a577be1435d412b47e3de2adfb2c2bbf58b180564b2f301f3523`.

## Command, artifacts, and cleanup

This command exited `0` from the isolated worktree. It made one unique Compose
project, ran PostgreSQL, Redis, and the two client containers, and removed its
containers, network, and volumes. A later `docker ps -a` filter for that
project returned no rows.

```sh
ROOT="$(git rev-parse --show-toplevel)"
"$ROOT/docs/research/experiments/e-hc-control/run.sh" \
  --output /tmp/lkjmc-e-hc-control-20260712
```

Ignored raw evidence remains at `/tmp/lkjmc-e-hc-control-20260712`. SHA-256
values are `compose.log`
`99f9e854df9504211dad1497d0f757c83566694db3d64225697110d4c61c29d0`,
`kubernetes.txt`
`5faa3b5378be50d1a828425ad24a88c6fe1d06d03c82a82327339b838b7e2b35`, and
`result.txt`
`fd252883fbdcff542d3c94ffe9bf910b8f7f0f6d7e530d06562247d3c78f6c51`.

## Results

| Probe | Result | Observed bounded evidence |
| --- | --- | --- |
| event reconstruction | PASS | Eight source rows accepted, a duplicate ID returned `false`, and rebuild produced `instance-alpha:running:4` and `instance-beta:stopped:4`. |
| two-daemon lease | PASS | A acquired generation 1; B was rejected while live, then after an 11-second real expiry acquired generation 2. A's stale write was `false`; B's fresh write was `true`. |
| PostgreSQL pull | PASS | The simple outbox claimed all 12 rows. An intentional `broker-gap` remained durable and the pull recovered that one row. |
| Redis Stream bridge | PASS | Twelve fixed IDs were added, consumer-group read created 12 pending entries, `XACK` returned 12, and pending reached zero. |
| Kubernetes lifecycle | BLOCKED | The harness attempted local access and recorded `kubectl is not installed`; no cluster, CRD, custom resource, or controller was claimed. |

The Redis bridge necessarily published after the PostgreSQL outbox write. The
unpublished gap therefore shows the bridge's unatomic boundary; it does not
measure throughput or prove delivery, ordering across a crash, or exactly-once
behavior. The generation check fenced only a guarded database write, not an
external effect issued by A before expiry.

## Deviations and external rerun

The retained readiness-only attempt at
`/tmp/lkjmc-e-hc-control-20260712-readiness-attempt` exited `1` before schema
setup because `pg_isready` did not guarantee a SQL-ready server; its log hash
is `83e0c0b20a018831f4c612359beb99488916e2cd5ffcf9640f03650b67230611`.
The retained lease-TTL attempt at `/tmp/lkjmc-e-hc-control-20260712-ttl-attempt`
exited `1`: a 300 ms lease elapsed during a real second container launch, so B
acquired generation 2 rather than proving rejection. Its log hash is
`662e346b43bc8d9b7dd4d1ee8dc1f26f804021d1e8272a9df26b0e958e198e2e`.
The final harness waits for `SELECT 1` and uses a 10-second lease; neither
incomplete attempt is candidate support.

Kubernetes remains external proof pending. With a legal disposable cluster,
rerun the command above with `LKJMC_KUBERNETES_SMOKE=1`,
`LKJMC_HC_KUBE_DISPOSABLE=1`, `LKJMC_HC_KUBE_NAMESPACE`, and
`LKJMC_HC_KUBE_CLUSTER_SCOPED=1`. The script then checks server access and
both permissions before creating and deleting its unique CRD and resource.
See the [decision](../decisions/e-hc-control-20260712.md) for disposition.
