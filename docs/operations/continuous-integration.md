# Continuous integration

## Purpose

Define fast and fresh-Compose CI lanes with retained structured evidence.

## Status

implemented

## Lanes

`verify.yml` runs on pushes to `main` and pull requests. `docs-contracts` runs
the fast owner checks. `verify-compose` exports the checked commit into a fresh
directory, builds pinned images without a host-language cache, and runs:

```sh
docker compose --profile verify run --rm verify
```

The Compose lane provides a fresh PostgreSQL database and runs
`./scripts/verify-full.sh`. Its final `ran` and `skipped` fields are captured as
structured JSON with the command and exit code. Required database probes cannot
skip. Live Minecraft, Discord, Bedrock, Kubernetes, installer-host, signing, and
public-network prerequisites stay explicit skips unless a separately authorized
lane actually supplies them.

## Failure and retention

No command is retried and no continue-on-error path can turn a failure into a
pass. Diagnostics run only after the original exit code is retained. CI uploads
the lane JSON, bounded redacted logs, test reports, artifact manifest, Compose
configuration, and cleanup result on success and failure. The secret-canary scan
runs before upload; tokens, database URLs, dumps, worlds, jars not in the release
manifest, and raw process logs are excluded.

The cleanup step always runs `docker compose down -v --remove-orphans` against
the unique project and checks that no project-labeled containers, networks, or
volumes remain. Cleanup failure fails the operations evidence even when tests
passed. Concurrency cancellation does not constitute a successful run.

## Local reproduction

Run `scripts/run-operations-lab.py --output /tmp/a-ops-evidence.json` from a
clean checkout. It performs two clean exports and two no-cache full Compose
runs, retains their separate outcomes, and cleans each project. A cached local
`target/`, Gradle home, Maven home, Docker build layer, or generated file must
not be required for success.
