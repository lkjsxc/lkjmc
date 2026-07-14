# Verification

## Purpose

This document defines verification tiers and guarded external checks.

## Status

implemented

## Tiers

| Tier | Command | Scope | Final line |
| --- | --- | --- | --- |
| Fast | `./scripts/verify-fast.sh` | docs, contracts, Rust checks, and tests without external services | `ok verify-fast skips=...` |
| Full | `./scripts/verify-full.sh` | fast scope, configured DB checks, JVM build, jar containment, and bounded checks | `ok verify-full ran=... skipped=...` |
| Default | `./scripts/verify.sh` | full wrapper | `ok verify-full ran=... skipped=...` |
| Live | `./scripts/verify-live.sh` | supported opt-in external checks | `ok verify-live ran=... skipped=...` |

Compose full verification uses:

```sh
docker compose --profile verify run --rm verify
```

## Supported live guards

| Smoke | Guard |
| --- | --- |
| Bedrock/Geyser | `LKJMC_BEDROCK_SMOKE=1`, enabled endpoint, and supported client |
| Discord | `LKJMC_DISCORD_SMOKE=1`, JSON config, credentials, and interaction access |
| Kubernetes | `LKJMC_KUBERNETES_SMOKE=1`, config, database URL, `kubectl`, credentials, and disposable namespace |

The Java Minecraft, claim, and playable paths are blocked diagnostics, not live
lanes. Setting their guards fails because Java daemon adapters are withdrawn.
`verify-live.sh` dispatches only the supported guards. A missing prerequisite is
a skip only when a supported lane was not attempted; a failed attempted lane is
never summarized as a pass.

## Truth probes

`./scripts/verify-truth-probes.sh` is an expected-failure harness. It proves
that known weak shapes and conforming-fixture mutations reject; it is not product
or adoption proof.

## Test-only fault harness

`./scripts/check-fault-harness.py --all` runs the Rust test-only fault scenarios
and release-artifact inspection. The script requires the daemon fault module to
be gated by `#[cfg(test)]`, requires its declared probes to remain selectable,
and rejects every fault marker from the release daemon and JVM artifacts. It
also proves no Java main/test source is required to stand in for a withdrawn
daemon adapter.

The fixed clock and deterministic schedule never sleep to create ordering.
Pre-commit faults leave a test transaction uncommitted; after-commit faults
record the durable test effect before returning the fault. Seed replay compares
a recorded same-seed transcript with a distinguishable different-seed transcript.
This harness falsifies test boundaries; it does not replace database,
child-process, or Minecraft proof.

## PostgreSQL test isolation

Every database test creates one random schema and a search-path-aware URL. That
same URL is used by direct clients and every `AppState` pool; migrations and
seeds therefore cannot use or reset `public`. Fixture cleanup drops only its
own schema, including after a failed assertion. Deadline-fault workers have a
unique `application_name`; fault inspection selects that exact backend rather
than a global lock row, so concurrent tests cannot cancel one another.

The full tier keeps Cargo's normal test parallelism. When its database URL is
configured, it also repeats daemon and store database tests with four test
threads as an isolation regression. The isolation runner obtains the exact
daemon binary test harness from Cargo's JSON compiler-artifact metadata. Only
one matching daemon binary target in the test profile is accepted; zero or
multiple matches fail. Hashed-file discovery is forbidden because normal and
test executables may coexist. Before concurrent execution, each named filter
must list at least one test. A deterministic regression supplies decoy hashed
executables and malformed metadata while proving only the selected harness runs.

The command-lifecycle and data-workflow harnesses fail closed when a named
database probe or ordinary aggregate lacks `LKJMC_STORE_TEST_DATABASE_URL`. A
host full-tier run without that URL explicitly uses aggregate allow-skip mode,
records each static data-workflow probe as run, and reports each skipped database
probe ID. With a real URL, the final full-tier `ran` list names all eight
individual data-workflow probes. The Compose full tier supplies that URL and
must not enable allow-skip mode. Ordinary Cargo tests may retain their documented
database skips; they are not task-probe success.

## Network adoption gate

`./scripts/check-network-adoption.py` exposes six exact, separately selectable
probes: `network-path-single`, `inspect-apply-pass`, `reapply-pass`,
`partial-failure-pass`, `local-kube-capabilities`, and `config-example-pass`.
The three persistence probes require a real PostgreSQL URL and cannot skip.
Their daemon integration boundary uses temporary local files and child
processes, restrictive generated files, bounded locking and deadlines,
connection release around process/readiness effects, durable observations,
no-op reapply, and recovery after injected partial failure. The recovery fault
matrix covers before effect, after config render, after child start, after
observation before the network-attempt commit, and daemon restart. It queries
old and retry attempts and proves marker-fenced adoption or intent-driven stop,
no false success, and no surviving child. A killed owned proxy must produce
drift, restart, and a new observation; stale or unowned identity must deny.
Filesystem, asset, readiness, and process blocking tests prove a size-one pool
remains available. Kubernetes denial is proved before an effect marker can be
written. Configuration examples run through the production parser and reject
placeholder, uniform, or repeated asset hashes.

The source audit inventories exact Rust planner/apply and runtime adapter
entrypoints, Java process creation, and shell launch sites. It requires the one
network intent-to-inspection compiler and rejects additional compilers, Java
launches, subprocess paths, or compatibility exports regardless of local names.
Checker mutation tests inject each forbidden path and prove rejection.

## Observability gate

`./scripts/check-observability.py` exposes six exact probes:
`correlation-pass`, `fault-diagnostics-pass`, `metrics-bounded`,
`support-bundle-pass`, `secret-canary-pass`, and `overhead-budget`. Correlation
uses the daemon HTTP router and fresh PostgreSQL 30 times. Support proof inspects
private archive and member modes, sorted manifest names, sizes, SHA-256 hashes,
and final redaction. Database probes fail when their URL is absent unless only
the aggregate full tier explicitly records them as skipped. `--mutations`
removes one required source marker per probe and requires all six to fail.

## Reviewer falsifiers

Reviewers should inject a renamed intent compiler, an indirect Rust process
spawn, Java `ProcessBuilder`, and shell Java launch; each must fail
`network-path-single`. They should block each external effect with pool size one
and concurrently acquire PostgreSQL. They should kill an owned proxy after a
successful apply and require drift, restart, and appended history. Replacing its
identity marker or introducing an unowned listener must deny without killing or
adopting that process. For observability, remove each checker-owned bound,
archive mode, final scan, or local-source disclaimer and require its named probe
to fail.

## Store and CLI gates

Store integration tests use the isolated-schema fixture and migrations. CLI
parsing has a Rust unit suite for command families and usage failures.
