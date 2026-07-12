# E-HC-CONTROL decision 2026-07-12

## Purpose

Interpret bounded high-cost research evidence without adopting a product
controller, event store, broker, replica daemon, Kubernetes path, or state
claim.

## Decision

**REJECT all tested adoption paths; leave Kubernetes external proof pending.**
The local event log reconstructs a small projection, and the two real Compose
clients fence one guarded database row. Neither result makes all domains event
sourced or fences an external effect. Redis Streams added a manual bridge and
consumer state while the PostgreSQL pull saw the deliberately unbridged durable
row; it is rejected as a simpler-candidate replacement. No candidate combines
with E-CONTROL's unresolved post-launch effect boundary.

The supporting [experiment](../experiments/e-hc-control-20260712.md) and
[Compose run](../runs/e-hc-control-20260712.md) use PostgreSQL and Redis only
inside a removed Compose project. The Kubernetes script attempted `kubectl` and
was blocked because it is absent. A CRD create/read/delete without a controller
would still be only a lifecycle observation, not reconciliation proof.

## Catalog dispositions

| Catalog ID | Disposition | Evidence or reconsideration condition |
| --- | --- | --- |
| `HC-EVENT-ALL` | REJECT | Eight-row reconstruction cannot establish all-domain event or external-effect semantics. |
| `HC-MULTI-DAEMON` | REJECT | Two clients fenced one database write, not an issued external effect or active replicas. |
| `HC-MULTI-REGION` | EXTERNAL PROOF PENDING | Needs controlled remote PostgreSQL regions and network shaping. |
| `HC-KUBE-OPERATOR` | EXTERNAL PROOF PENDING | `kubectl` attempt blocked; needs authorized namespace and cluster-scoped CRD approval. |
| `HC-BROWSER-SPA` | NOT EVALUATED; NO ADOPTION | Needs a bounded authenticated operator journey. |
| `HC-PLUGIN-SDK` | NOT EVALUATED; NO ADOPTION | Needs extension isolation, versioning, and revoke evidence. |
| `HC-LANGUAGE-REWRITE` | NOT EVALUATED; NO ADOPTION | Needs a measured replacement boundary and migration proof. |
| `HC-EMBEDDED-STORE` | REJECT | Conflicts with PostgreSQL durable-truth boundary. |
| `HC-MESSAGE-BROKER` | REJECT | Redis bridge has a pre-publish gap; PostgreSQL pull is the simpler observed candidate. |
| `HC-GRAPHQL` | NOT EVALUATED; NO ADOPTION | Needs authorization and query-cost evidence. |
| `HC-WASM-RULES` | NOT EVALUATED; NO ADOPTION | Needs sandbox escape, fuel, and policy rollback evidence. |
| `HC-AI-OPERATOR` | NOT EVALUATED; NO ADOPTION | Needs recorded incidents with no mutation authority. |
| `HC-PUBLIC-CONTROL` | NOT EVALUATED; NO ADOPTION | Needs local authentication, abuse, and revocation evidence. |
| `HC-MOBILE-CLIENT` | NOT EVALUATED; NO ADOPTION | Needs an authenticated local operator journey. |
| `HC-PREDICT-WAKE` | NOT EVALUATED; NO ADOPTION | Needs controlled workload and false-wake cost data. |
| `HC-WORLD-SERVICE` | EXTERNAL PROOF PENDING | Needs a controlled remote storage endpoint and fault shaping. |

## Limits and next step

The event probe omitted concurrent append conflict, retention, authorization,
and effects. The lease omitted daemon recovery, heartbeats, split brain after a
network partition, and adapter idempotency. The broker probe omitted retries,
back pressure, retention, and failure between Redis acknowledgement and the
consumer's external effect. Kubernetes access, ownership, observation, and
recovery are unproved.

Do not create a controller or owner/state row from this decision. Reconsider a
specific candidate only with a separate hypothesis that closes its listed
limit. The next executable step is to provide an authorized disposable
Kubernetes cluster, run the retained lifecycle script with all four flags, and
record its cleanup and blocked-or-observed result.
