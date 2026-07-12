# E-HC-PLATFORM decision 2026-07-12

## Purpose

Interpret bounded research evidence without adopting a high-cost platform path.

## Disposition

Research only: no high-cost catalog candidate is adopted, registered, or
supported. PostgreSQL remains the product durable truth; no controller, state,
owner contract, command, adapter, policy authority, or deployment changes.

Evidence is the [hypothesis](../experiments/e-hc-platform-20260712.md),
[run](../runs/e-hc-platform-20260712.md), prior harness commit
`b1052013fa49277218d06b64222d0c8c741c66c9`, and source-controlled
[capture anchor](../experiments/e-hc-platform-20260712.capture.json). It binds
capture tip `f9203e1d43d6da52ad29ad75fed0282ac20f5f42`, harness hash
`18237367461e07c33eda16a35df82b5503b86105bb9c5417a978e2c819e2f866`, and raw
index `390e35fc2226657a0ccfe625125953f946736c75d037514d966977813b6dc193`.

## Catalog dispositions

| Catalog ID | Current disposition | Reason / reconsideration |
| --- | --- | --- |
| `HC-EVENT-ALL` | deferred; no adoption | No bounded journal need was run. |
| `HC-MULTI-DAEMON` | deferred; no adoption | No replica fencing or recovery slice was run. |
| `HC-MULTI-REGION` | deferred; no adoption | No controlled distant database endpoint was supplied. |
| `HC-KUBE-OPERATOR` | deferred; no adoption | No authorized disposable namespace was supplied. |
| `HC-BROWSER-SPA` | deferred; no adoption | No bounded operator journey was run. |
| `HC-PLUGIN-SDK` | deferred; no adoption | No extension isolation or compatibility slice was run. |
| `HC-LANGUAGE-REWRITE` | not adopted | Rust-to-Python adds contract, child, packaging, and observability ownership; parity is unmeasured. |
| `HC-EMBEDDED-STORE` | not adopted | Private SQLite lock/backup evidence cannot displace PostgreSQL truth. |
| `HC-MESSAGE-BROKER` | deferred; no adoption | No delivery bound exceeding PostgreSQL/HTTP was run. |
| `HC-GRAPHQL` | deferred; no adoption | No bounded control-surface query was run. |
| `HC-WASM-RULES` | not adopted | Narrow no-import/Wasm permission evidence has no product authority or host lifecycle proof. |
| `HC-AI-OPERATOR` | deferred; no adoption | No mutation-free incident corpus was run. |
| `HC-PUBLIC-CONTROL` | deferred; no adoption | No local public-control authorization slice was run. |
| `HC-MOBILE-CLIENT` | deferred; no adoption | No operator client journey was run. |
| `HC-PREDICT-WAKE` | deferred; no adoption | No historical demand data or false-wake cost was run. |
| `HC-WORLD-SERVICE` | blocked; no adoption | Endpoint unset; the run records `urlopen`/request count zero and its exact prerequisite and rerun. |

“Deferred” is not a rejection based on absent evidence; it is an explicit
no-adoption disposition for this task. Only the four representative candidates
received an E-HC execution, and none becomes a product proposal from it.

## Comparison and decision basis

The alternate boundary’s 30 one-child requests and missing-child failure show
extra integration ownership, not a language performance winner. The private
SQLite slice demonstrates local lock and restore mechanics but also introduces
second-store backup, schema, and ownership duties; it cannot establish
multi-process consistency, failover, or controller behavior. The zero-import
Wasm module returned allow/deny while Node permission mode denied a host file
write. That reduces the module’s immediate capability but does not establish a
stable runtime, authorization source, audit, timeout, upgrade, or escape model.

The remote-world lane is explicitly `BLOCKED`: the required controlled endpoint
variable was absent, with `urlopen`/request count zero. Replay preserves those
fields and rejects checksum-recomputed reordered, missing, extra, and
content-forged artifacts against the immutable source anchor, not the raw
manifest. The current correction source post-dates the capture and does not
claim to have generated it. It supplies no latency, throughput, consistency,
durability, network-fault, or deployment evidence.
No unavailable external lane is counted as passed.

## Reconsideration and next step

A language candidate needs one current adapter workflow with versioned contract,
child lifecycle, cancellation, overload, tracing, rollback, and independent
operational comparison. An embedded-store candidate needs a failure-consistent
product ownership proposal that preserves PostgreSQL truth and measures backup,
restore, concurrent access, corruption, and recovery.

A policy candidate needs a supported sandbox runtime, explicit capabilities,
resource/time limits, signed module lifecycle, audit/revocation, and a real
fail-closed product boundary. A world-service candidate needs an authorized
controlled endpoint, network shaping, cleanup confirmation, byte/durability
checks, and incident recovery. The next executable step is to provision that
endpoint and rerun only the documented remote lane; it remains research-only.
