# E-HC-AUTOMATION decision 2026-07-12

## Purpose

Interpret the bounded [run](../runs/e-hc-automation-20260712.md) without
adopting a product feature, command, deployment model, support promise, or
controller task.

## Dispositions

| Candidate | Disposition | Evidence boundary |
| --- | --- | --- |
| `HC-AI-OPERATOR` | Reject for adoption; retain offline pattern only | Five sanitized records generated references, not effects. There is no production corpus, authority model, audit design, or validation of a remediation result. |
| `HC-PREDICT-WAKE` | Reject for adoption | Fixture arithmetic reduced modeled delay 135 to 45 seconds but made one false prewarm from three eligible predictions; one unknown-presence row was skipped. No player, queue, runtime, cost, or calibration evidence exists. |
| `HC-MULTI-REGION` | External proof pending | Offline-only local PostgreSQL observed 76 ms server delay and a 25 ms timeout. The local image was preflighted and Compose used `--pull never`; the regional record remains a model and isolated `tc netem` was denied. |

## Safety and cost comparison

The offline classifier emits one of four incident-runbook references or no
recommendation, each with `operator-review`. It has no command transport,
credentials, process handle, database connection, or mutation authority. That
is necessary for the experiment but insufficient for automated remediation.

Reactive wake has the current durable queue and runtime observation boundaries.
The predictor replay does not model queue expiry, start failure, active session,
stale heartbeat, actual player arrival, memory cost, or a real join. A lower
fixture delay therefore cannot justify a prewarm behavior or change the rule
that unknown presence is skipped.

The revised raw manifest cited by the run hashes fixture/model inputs and image
preflight, Compose, and netem logs; replay checks that coverage and committed
input hashes. The PostgreSQL lane preflighted a local image and started its
unique container with `--pull never`, then removed its resources. Its delay is
inside one database server, not a remote network. The denied `unshare` message
leaves WAN latency, loss, recovery, replicas, consistency, and failover unproved.

## Reconsideration

Reconsider the offline recommender only with a redacted, authorized incident
corpus; reviewed runbook-to-action authority; explicit human approval; durable
audit and rollback evidence; and independent outcome evaluation. Reconsider
predictive wake only with opt-in, bounded real queue/runtime observations,
false-positive cost, player arrival evidence, and all existing presence safety
rules. Reconsider multi-region only with authorized disposable endpoints,
actual shaping or a recorded equivalent, failure/recovery evidence, and database
consistency analysis.

No owner or state document changes, runtime registration, deployment support,
or adoption task follows from this decision. The next executable step is to run
the emitted netem command in an authorized disposable namespace and record its
result as external proof; otherwise leave this candidate blocked.
