# E-HC-PLATFORM run 2026-07-12

## Purpose

Record a bounded high-cost platform comparison without adopting a language,
store, policy runtime, remote-world service, product behavior, or deployment
support.

## Identity and reproducibility

- Hypothesis: [E-HC-PLATFORM experiment](../experiments/e-hc-platform-20260712.md).
- Base: `4b9357a`; prior harness commit: `b1052013fa49277218d06b64222d0c8c741c66c9`.
- Host: Linux; Python `3.12.3`; rustc `1.96.0`; Node `22.22.3`; seed `20260712`.
- Capture tip: `f9203e1d43d6da52ad29ad75fed0282ac20f5f42`; harness SHA-256:
  `18237367461e07c33eda16a35df82b5503b86105bb9c5417a978e2c819e2f866`.
  The capture anchor was committed after that tip, so the current correction
  harness is not claimed to have generated this raw evidence.
- Correction raw root: `/tmp/lkjmc-e-hc-platform-artifact-manifest-correction-20260712`,
  owned mode `0700`; raw index SHA-256:
  `390e35fc2226657a0ccfe625125953f946736c75d037514d966977813b6dc193`.
  The source-controlled [capture anchor](../experiments/e-hc-platform-20260712.capture.json)
  records this index digest, manifest digest
  `c025d776c3e11f7147795cf5a02ac7df44d98e85c89a82e278b19353c9712174`,
  five capture-tip source hashes, and all 43 ordered path/size/content hashes.

```sh
REPO="$(git -C /path/to/lkjmc rev-parse --show-toplevel)"
python3 "$REPO/docs/research/experiments/e-hc-platform-20260712.py" replay \
  --output /tmp/lkjmc-e-hc-platform-artifact-manifest-correction-20260712
python3 "$REPO/docs/research/experiments/e-hc-platform-20260712.py" self-test \
  --output /tmp/lkjmc-e-hc-platform-artifact-manifest-correction-20260712
```

The capture run, anchored replay, and self-test exited `0`. Replay compares
raw index, ordered list, and every content hash to the source anchor rather
than trusting the raw manifest; it rejects undeclared files. End-to-end tests
recompute raw index and manifest after reordered, missing, extra, or
content-forged artifacts, and each fails. The runner derives its root, refuses
pre-existing/non-`/tmp` outputs, and cleanup removes only marker-owned output.

## Results

| Slice | Result | Observation |
| --- | --- | --- |
| Rust to Python boundary | PASS | 30 serial JSON-line requests had p50 `11.548 ms`, p95 `12.619 ms`; a missing child exited `1`, with no fabricated action. |
| private SQLite slice | PASS | One revision persisted through backup/restore; a conflicting writer was locked; the single whole-slice sample was `68.948 ms`. |
| WebAssembly policy | PASS | `wasm32-unknown-unknown` build had zero imports; Node permission mode allowed the two reads, allowed valid/denied malformed input, and denied a write probe. |
| remote-world HTTP I/O | BLOCKED | `LKJMC_REMOTE_WORLD_URL` was unset; `urlopen`/request count was zero, so no remote request or latency measurement occurred. |

The boundary duration includes a new Python child per request and is not daemon,
plugin, database, or production latency. The SQLite duration includes setup,
the intentional lock timeout, backup, and restore; it is not a throughput or
availability comparison. The Wasm check does not prove a container boundary,
network-egress control for a host, policy authorization, or a deployment.

## Costs observed, not syntax preferences

| Candidate | Integration cost | Operational cost and limit |
| --- | --- | --- |
| language rewrite | Rust binary, Python runtime, JSON contract, child failure handling, packaging, and correlation/logging must all be operated. | No baseline parity, restart, upgrade, or incident evidence exists. |
| embedded truth | A second schema, lock semantics, backup/restore, retention, and split ownership must be maintained. | Private SQLite cannot replace PostgreSQL durable truth from this one-process slice. |
| Wasm rules | A target, module ABI, no-import review, Node permission host, and deny-default parser are extra boundaries. | Node permission mode is experimental; only its file-write denial and module capability absence were measured. |
| world service | Object protocol, identity, cleanup, network shaping, retries, consistency, and observability need an operator-owned endpoint. | No controlled endpoint was supplied, so no remote cost or support conclusion exists. |

The private database hashes are `276a8ed4209022b8fd30ed50c2b44c767d46ce9c9ee0572b5593760c061c2e56`
and `7b1bd17b52dae17cd2bda89c19ccd8016830e143989b1d8018dfc2fdf734aed3`;
the generated Wasm hash is `a454e504a437f556a3670ce2c1a821032c31a83fc8da58ded140b42bf77cf1ad`.
They are bounded research artifacts, not product data.

## Exact remote-world block and cleanup

The retained `remote-world-attempt.log` is:

```text
blocked before network access: LKJMC_REMOTE_WORLD_URL is unset; urlopen/request count=0
rerun=LKJMC_REMOTE_WORLD_URL=<controlled-url> python3 docs/research/experiments/e-hc-platform-20260712.py --output /tmp/lkjmc-e-hc-platform-remote
```

With a controlled endpoint, the harness performs ten 64 KiB byte-equal HTTP
`PUT`/`GET` pairs and deletes each random object; any network or cleanup error
is external pending. It neither prints the configured URL nor creates an
endpoint. No product/controller/state file, migration, command, or external
world was changed. Interpretation is in the
[decision](../decisions/e-hc-platform-20260712.md).
