# E-HC-AUTOMATION offline automation comparison

## Purpose

Compare three high-cost research candidates without registering a command,
starting a product runtime, modifying a product database, or granting any
remediation authority.

## Candidates and baseline

This bounds `HC-AI-OPERATOR`, `HC-PREDICT-WAKE`, and `HC-MULTI-REGION` from
[high-cost alternatives](../ideas/high-cost.md). The implemented baseline is
operator evidence-first response, presence-driven autosuspend, and queued
reactive wake. In particular, unknown or stale presence must not autosuspend,
and a wake request records a queue row before a runtime start is attempted.

## Hypotheses and invariant

A deterministic offline classifier can emit only runbook references for a
sanitized incident record; it must never emit a product command, token, or
runtime effect. Replaying known-empty presence can compare predicted prewarm
with reactive wake, while unknown presence remains skipped. A fixed regional
latency/failure record can be loaded into a disposable PostgreSQL container;
server delay and timeout are database observations, not WAN measurements.

The harness reads only its committed fixture and writes raw evidence under one
owned `/tmp` root. Before Compose, it inspects the local `postgres:16-alpine`
image; if absent, the PostgreSQL lane is explicitly `BLOCKED` and does not
start. Its uniquely named Compose container starts only with `--pull never` and
is removed with volumes. It uses no product daemon, CLI, migration, persistent
volume, external endpoint, or controller update.

## Workload and measurements

The fixed fixture has five sanitized research incident records, four wake
windows, and eight regional attempts. The recommender must map four known
symptoms to `operator-review` references and leave the unknown symptom with no
recommendation. The replay records reactive and predicted join-delay totals,
hits, false prewarms, and unknown-presence skips. It records p95 successful
latency and failures per modeled region.

The PostgreSQL lane requires a locally present image, then loads the eight
attempts, observes `pg_sleep(.075)`, and expects a 25 ms `statement_timeout`.
Its raw coverage hashes the fixture, model, image preflight, every executed
Compose command, and isolated `tc netem` attempt. Success configuring a private
namespace would not prove a remote region; denied setup is `BLOCKED`, never a
model pass.

## External proof and rerun

The repository contains no production incident corpus: the operations runbook
requires incident records outside the repository. The committed fixture is a
sanitized research record, so incident representativeness is `BLOCKED`. Remote
multi-region proof also requires authorized disposable endpoints and network
administrative capability.

```sh
REPO="$(git rev-parse --show-toplevel)"
SCRIPT="$REPO/docs/research/runs/e-hc-automation-20260712.py"
(cd /tmp && python3 "$SCRIPT" run)
(cd /tmp && python3 "$SCRIPT" replay --raw-dir <emitted-root>)
```

The [run](../runs/e-hc-automation-20260712.md) and
[decision](../decisions/e-hc-automation-20260712.md) retain the observation and
non-adoption dispositions. Base commit: `4b9357a8e1a7949e0ebfe59c16af5196554f46cc`.
