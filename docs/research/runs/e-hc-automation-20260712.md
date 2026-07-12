# E-HC-AUTOMATION run 2026-07-12

## Purpose

Record a bounded offline observation. It neither authorizes remediation nor
adds predictive wake, multi-region deployment, external support, or adoption.

## Identity and reproduction

- Experiment: [E-HC-AUTOMATION](../experiments/e-hc-automation-20260712.md).
- Base: `4b9357a8e1a7949e0ebfe59c16af5196554f46cc`; seed is the committed fixture.
- Harness SHA-256: `27f4ee7873c63c95dc4f05a075846ae6d149cec68e649a2e2fdcb07a0b7bed3e`.
- Fixture SHA-256: `77331bad153ea36eb30e90eb8eee156406ef6fb870a07a87c023277354d16211`.
- Environment: Python 3.12.3; Docker 29.5.3; Docker Compose 5.1.4.
- Raw root: `/tmp/lkjmc-e-hc-automation-259e0b98f8339246f6c2e7118d755856`.
- Index SHA-256: `c49ac683608b9df87c05289da1ed33a2e76fc044eee9b1597f29dfd2339e16d1`.

```sh
(cd /tmp && python3 "$REPO/docs/research/runs/e-hc-automation-20260712.py" run)
(cd /tmp && python3 "$REPO/docs/research/runs/e-hc-automation-20260712.py" replay \
  --raw-dir /tmp/lkjmc-e-hc-automation-259e0b98f8339246f6c2e7118d755856)
```

The executed replay returned `E-HC-AUTOMATION replay=PASS`. The revised raw
`index.json` hashes the retained fixture, offline model, and each covered command log:
image preflight, Compose up/model/delay/timeout/down, and netem. It also checks
the fixture and model against committed inputs. The root is ignored and owned,
not a committed log or product artifact.

## Offline recommender and wake replay

All five records produced `operator-review`: daemon unavailable mapped to stop
dependent mutations and inspect; database error to hold writes and inspect;
backend unavailable to hold transfers and inspect; suspected exposure to revoke
access and inspect; unclassified to no recommendation. The harness issued no
command and contained no action implementation.

| Wake comparison | Value |
| --- | ---: |
| Reactive join delay | 135 seconds |
| Predicted join delay | 45 seconds |
| Known-empty eligible predictions / hits | 3 / 2 |
| False prewarms | 1 |
| Unknown-presence skips | 1 |

These are fixture arithmetic, not player, queue, runtime, capacity, or
predictor-accuracy evidence. The unknown row was not prewarmed, consistent with
the implemented presence safety boundary.

## Model and disposable PostgreSQL

| Model region | Attempts | Failed | p95 successful latency |
| --- | ---: | ---: | ---: |
| same-region | 3 | 0 | 14 ms |
| near-region | 3 | 1 timeout | 103 ms |
| far-region | 2 | 1 connection refusal | 181 ms |

`docker image inspect postgres:16-alpine` exited 0 before Compose. A unique
Compose PostgreSQL 16 container then started with `--pull never`, reached
healthy state, loaded eight rows into the untracked `hc_automation_replay` table,
and removed its container, network, and volumes. `pg_sleep(.075)` measured 76
ms. With a 25 ms statement timeout, PostgreSQL exited 1 with `canceling
statement due to statement timeout`. If the local image is absent, the lane is
explicitly `BLOCKED` and Compose does not start. This is local database
delay/failure evidence, not regional latency or a topology claim.

## Blocked shaping and external evidence

The isolated attempt exited 1 before shaping:

```text
unshare: write failed /proc/self/uid_map: Operation not permitted
```

Thus `network-shaping=BLOCKED`. The missing proof is permission for a disposable
user/network namespace with `tc netem`, plus authorized controlled remote
endpoints. The exact emitted rerun is in raw `index.json`; no external endpoint
or product network was contacted. The repository also has no production incident
records, so the committed sanitized fixture cannot establish incident coverage.

See the [decision](../decisions/e-hc-automation-20260712.md) for dispositions.
