# Continuous integration

## Purpose

Define fast and fresh-Compose CI lanes with retained structured evidence.

## Status

implemented

## Lanes

`verify.yml` runs on pushes to `main` and pull requests. `docs-contracts` runs
the fast owner checks. `verify-compose` exports the checked commit into a fresh
directory, builds pinned images without a host-language cache, and bounds Rust
test concurrency at four workers so database lock demand is host-independent.
It runs:

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
pass. Diagnostics run only after the original exit code is retained. On success CI uploads the lane JSON, bounded redacted logs, release manifest
and sidecar, resolved and redacted Compose configuration, cleanup result, and a
checksum/size index. Each class has a documented maximum size and exact retained closure. Evidence
preparation rejects any unreadable old-style raw root and accepts only a private,
deterministically traversed input closure. Its index set equals every retained
regular output file; unindexed contents are fatal. A generated random canary
plus credential-value scan covers the full
source context, release, saved image layers, and retained evidence before any
upload. The saved verifier image receives the same 2 GiB regular-tar audit as
the local lab: bounded canonical members, Docker manifest/config closure,
digest agreement, and one content check per distinct declared layer. Shared
references are valid; missing, unreferenced, conflicting duplicate, special,
traversing, or oversized members fail. Safe literal parameter names, printf URL
placeholders, bounded tool examples, and diagnostic prose are not findings. Nested image-layer
links are skipped rather than materialized; links in authored archives fail. The upload step is gated
on recorded scan success, not `always()`. On scan failure CI uploads only a
constant safe failure marker, never the rejected bundle. Cleanup still always
runs; dumps, worlds, undeclared jars, raw process logs, and unbounded reports
are never retained.

Evidence and secret traversal is descriptor-relative and no-follow. Every entry
must remain the same no-follow identity when opened, regular files and
directories must have private deterministic modes, and symlinks, special files,
unreadable entries, traversal races, device/root crossing, and count, byte, or
depth overflow fail the lane. The unreadable canary falsifier drops privileges;
root execution cannot turn a `000` directory into a pass.

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
