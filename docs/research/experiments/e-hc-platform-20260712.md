# E-HC-PLATFORM high-cost platform experiment

## Purpose

Measure operational and integration boundaries for four costly alternatives
without changing product/store/controller state or registering a capability.

## Catalog and baseline

This evaluates `HC-LANGUAGE-REWRITE`, `HC-EMBEDDED-STORE`, `HC-WASM-RULES`,
and `HC-WORLD-SERVICE` from the [high-cost catalog](../ideas/high-cost.md).
The shipped baseline remains a Rust control plane with PostgreSQL durable truth;
this is not a proposal to replace either. The remaining catalog IDs receive an
explicit no-adoption disposition only after the run records their evidence gap.

## Hypothesis and representative slices

A Rust caller invokes a separate Python policy process over one JSON-line
request/response boundary. That makes serialization, process lifetime, failure
semantics, packaging, and observability costs measurable instead of comparing
language syntax. A private SQLite file stores one product-shaped desired-state
revision, performs a conflicting transaction, and verifies a backup/restore.
It is not product truth or a migration candidate.

A no-import WebAssembly policy module is built for `wasm32-unknown-unknown`
and run by Node's experimental permission mode. The runner grants read access
only to itself and the module, then proves a host write probe is denied. A
missing Rust target or Node permission support is a blocked sandbox lane; a
resource limit or an in-process callback alone is never called a sandbox. The
module must deny malformed input and produce no effect.

The remote-world lane requires a controlled `LKJMC_REMOTE_WORLD_URL` endpoint
that accepts authenticated-free test `PUT` and `GET` of a disposable named
object. The runner first records the exact configured endpoint attempt without
printing query credentials. With no endpoint, it records an explicit `BLOCKED`
lane with `urlopen`/request count zero, the harness, and rerun command. A
loopback HTTP calibration, if run, is not remote-world proof.

## Invariants and prohibited behavior

- Writes are limited to the harness-owned ignored `/tmp/lkjmc-e-hc-platform-*`
  root and `docs/research/**`; no product, controller, state, or migration file
  is written.
- Fixed input is JSON; no secret, credential, or endpoint query is retained.
- The alternate boundary accepts one request, rejects malformed output, and
  records child failure rather than fabricating an action.
- SQLite holds only private disposable data; backup restoration must preserve
  the one revision and a lock must reject the conflicting write.
- A sandbox pass requires all declared container restrictions plus deny result;
  the module has no authority to mutate a product or remote world.
- A remote-world pass requires both exact `PUT` and subsequent byte-equal `GET`
  against the configured controlled endpoint. Network failure is pending, not
  a local performance result or deployment support.

## Workload, faults, and measurements

The seed is `20260712`; the alternate boundary runs 30 serial requests after
one warm-up. SQLite inserts revision `1`, holds `BEGIN IMMEDIATE` while a second
writer times out, then copies and restores the database. The policy runs one
allow-shaped and one malformed request. The world request uses a random object
name and 64 KiB deterministic payload with 10 requests per method when enabled.
The runner records duration samples, p50/p95, exit codes, byte checks, tool
versions, capped sanitized logs, hashes, and an index. The source-controlled
[capture anchor](e-hc-platform-20260712.capture.json) records the capture tip,
harness hash, raw index/manifest digests, and exact ordered artifact metadata.
Replay compares raw index, list, and content to that anchor, never treating the
raw manifest as authority. The correction source is after the capture tip and
is not claimed to have generated the raw. Self-test recomputes raw index and
manifest checksums after reordered, missing, extra, and content-forged artifacts;
each must fail. The `BLOCKED` remote lane still requires no request fields.

## Scope, prerequisites, and rerun

Base commit is `4b9357a`; this worktree is isolated. Python 3, `rustc`, the
installed `wasm32-unknown-unknown` target, and Node permission support are
local prerequisites. The remote lane additionally needs the operator-provided
controlled endpoint. No endpoint setup or cloud resource is implied by this
hypothesis.

```sh
python3 "$REPO/docs/research/experiments/e-hc-platform-20260712.py" replay \
  --output /tmp/lkjmc-e-hc-platform-artifact-manifest-correction-20260712
```

A new run is a new observation, not replay of this capture; commit a new anchor
before calling it authenticated evidence. A controlled endpoint remains a
prerequisite for the remote lane and is never provisioned by this harness.

The [run](../runs/e-hc-platform-20260712.md) and
[decision](../decisions/e-hc-platform-20260712.md) retain outcomes. This
hypothesis changes no owner contract and never represents deployment support.
