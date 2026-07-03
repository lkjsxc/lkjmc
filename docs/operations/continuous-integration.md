# Continuous integration

## Purpose

This document owns the GitHub Actions verification contract.

## Triggers

The `verify.yml` workflow runs on pushes to `main` and on every pull request.
Only one run per Git ref stays active; newer pushes cancel older runs for the
same ref.

## Gate

CI runs the same full gate that agents run locally:

```sh
docker compose --profile verify run --rm verify
```

That Compose gate starts PostgreSQL, injects `LKJMC_STORE_TEST_DATABASE_URL`,
and runs `./scripts/verify-full.sh` inside the verify image. It covers docs,
contract checks, Rust formatting, clippy, Rust tests, Java tests, plugin jar
assembly, and guarded smoke wrappers.

## Caching and network

The workflow uses Docker Buildx with GitHub Actions cache for the verify image.
Cached dependency layers reduce repeated Cargo, Gradle, and system package work.
The GitHub-hosted runner allows outbound dependency resolution, so Gradle Paper
and Velocity dependencies resolve during the normal container verify path rather
than through a vendored CI-only path.

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
