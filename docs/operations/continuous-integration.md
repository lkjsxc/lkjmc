# Continuous integration

## Purpose

This document owns the GitHub Actions verification contract.


## Status

implemented

## Triggers

The `verify.yml` workflow runs on pushes to `main` and on every pull request.
Only one run per Git ref stays active; newer pushes cancel older runs for the
same ref.

## Gates

CI has a fast `docs-contracts` lane for line, docs, command, permission, locale,
menu, and config drift checks. The `verify-compose` lane keeps the full gate:

```sh
docker compose --profile verify run --rm verify
```

That Compose gate starts PostgreSQL, injects `LKJMC_STORE_TEST_DATABASE_URL`,
`LKJMC_CLAIM_SMOKE=1`, and `LKJMC_WEB_SMOKE=1`, then runs
`./scripts/verify-full.sh` inside the verify image. It covers Rust formatting,
clippy, Rust tests, Java tests, plugin jar assembly, and deterministic claim
and private-web smokes after the contract checks. Claim requests require one
nonempty JSON daemon response; a startup, transport, or response failure fails
with a redacted diagnostic instead of being treated as a passing smoke.
Remaining guarded lanes are listed as exact skips in the final summary. Because
PostgreSQL is present, this lane runs command-lifecycle probes without database
skip permission; any required probe skip fails the gate.

## Caching and network

The workflow uses Docker Buildx with GitHub Actions cache for the verify image.
Cached dependency layers reduce repeated Cargo, Gradle, and system package work.
The GitHub-hosted runner allows outbound dependency resolution, so Gradle Paper
and Velocity dependencies resolve during the normal container verify path rather
than through a vendored CI-only path. CI uploads test reports as artifacts on
failure or success without including tokens, databases, logs, worlds, or `tmp/`.

## Live smoke policy

CI never enables live smoke prerequisites such as Minecraft EULA acceptance,
Discord credentials, Kubernetes access, installer host mutation, or live network
download flags. Guarded live smokes must print skipped when prerequisites are
absent; a skip is not reported as a pass.

## Local reproduction

To reproduce a CI failure, run the exact Compose gate above from a clean worktree.
If a failure depends on cached state, remove the Compose project first:

```sh
docker compose --profile verify down -v
```
