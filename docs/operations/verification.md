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
threads as an isolation regression.

The command-lifecycle harness fails closed when a named database probe or its
ordinary aggregate lacks `LKJMC_STORE_TEST_DATABASE_URL`. A host full-tier run
without that URL explicitly invokes the aggregate's allow-skip mode and reports
each skipped database probe ID. The Compose full tier supplies a real URL and
must not enable allow-skip mode. Ordinary Cargo tests may retain their documented
database skips; they are not task-probe success.

## Store and CLI gates

Store integration tests use the isolated-schema fixture and migrations. CLI
parsing has a Rust unit suite for command families and usage failures.
