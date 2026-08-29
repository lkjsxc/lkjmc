# Continuous integration

## Purpose

Define fast and fresh-Compose CI lanes with retained structured evidence.

## Status

implemented

## Lanes

`verify.yml` runs on pushes to `main` and pull requests. `docs-contracts` runs
the fast owner checks. `verify-compose` fetches complete Git history, requires
the checked `HEAD` to equal the workflow commit, and exports that object into a
fresh directory. Its source bundle advertises only
`refs/bundles/lkjmc-source`; the producer independently imports and checks out
that ref in an empty repository before the bundle can be consumed. The lane
then builds pinned images without a host-language cache and bounds Rust
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
placeholders, bounded tool examples, and diagnostic prose are not findings. Nested image-layer links are inspected as metadata and never materialized;
authored archive links fail. Filesystem-tree links always fail. The upload step is gated
on recorded scan success, not `always()`. On scan failure CI uploads only a
constant safe failure marker, never the rejected bundle. Cleanup still always
runs; dumps, worlds, undeclared jars, raw process logs, and unbounded reports
are never retained.

The Docker-save parser validates the format's optional parent and layer-source
metadata instead of treating those documented fields as undeclared files.
Parents must name another retained image config. Layer-source keys must name an
actual declared diff ID; their media type, digest, size, annotations, and absence
of external URLs are checked. Docker may retain a registry's multi-platform OCI
index while exporting only the selected platform. An absent descriptor is
therefore accepted only as a child of a retained index. Every retained
descriptor must still have the declared size and digest, every retained manifest
must be complete, and every Docker config and layer must occur in that complete
retained OCI closure. Docker's classic image store also emits one synthetic
legacy config for each layer prefix. Those otherwise-unreferenced blobs are
accepted only when their content-addressed files form the unique legacy
`id`/`parent` chain for the declared diff-ID order and the terminal config
matches the retained image config after Moby's typed V1 serializer adds only
explicit zero-value container-config fields. A missing, altered, ambiguous,
detached, nonzero-expanded, or schema-expanded legacy config fails; every other
extra blob remains fatal.

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

Release construction imports the sole advertised source-bundle ref into the
Git-less export, then builds twice into separate fresh roots inside the same
exact verifier image. `scripts/compare-release-roots.py` uses the bounded
descriptor-safe walker and requires equal path, type, mode, size, and SHA-256
closure before either root can become retained evidence. The second root exists
only inside the `--rm` verification container. A host build using a different
native linker or C library is a different environment, not contradictory proof
about the pinned verifier image.

After equality, `verify-compose` packages the first accepted root twice with
`scripts/release_archive.py` and compares both three-file handoff closures. It
safely extracts the canonical copy, runs the existing manifest and embedded
binary/JAR identity verifiers, and compares the extracted root back to the
accepted input. The release handoff is secret-scanned with the source context,
release root, saved image, and bounded operations evidence. Its upload runs
only when every earlier producer result, exact evidence preparation, cleanup,
and scan succeeded. The artifact name includes commit, workflow run, and
attempt; overwrite is disabled and retention is explicitly 30 days.

Operations evidence remains the exact indexed diagnostic artifact described
above. Release bytes are a different artifact containing only the canonical
tar, archive sidecar, and handoff descriptor. Neither closure is called the
other, and the release root itself is never uploaded unpacked.

The separate required `verify-release-artifact` job depends on successful
`verify-compose`. It checks out the same commit only to obtain the canonical
verifiers, downloads the exact artifact ID with the pinned download action,
and separately retrieves the raw outer ZIP through the artifact API. It
requires the service metadata's ID, name, run, commit, size, expiry state, and
SHA-256 to agree with producer outputs. It then requires the exact inner
three-file closure, safely consumes the archive, independently verifies the
manifest and embedded Rust/JVM identities without Cargo metadata, compilation,
Gradle, or release reconstruction, confirms extraction cleanup, scans the
download and receipt, and retains the one-file consumer receipt separately.
The maintained JVM artifacts target Java 21, so the consumer does not trust the
hosted runner's mutable default Java. It explicitly selects the runner's
preinstalled `JAVA_HOME_21_X64` only for identity execution. It neither installs
a JDK nor resolves or rebuilds release content.
Any download, outer digest, descriptor, sidecar, archive, extraction,
manifest, identity, cleanup, receipt scan, or receipt upload failure fails the
workflow.

The upload and download action references are immutable commits. The pinned
upload action reports artifact ID and service digest and refuses collisions;
the pinned download action supports exact same-run ID selection. GitHub's
outer transport mode normalization is not trusted. The permission-preserving
inner tar and its independently recomputed digest are the handoff identity.
Consumer success is `RELEASE ARTIFACT VERIFIED`, not installed, running,
ready, player-accessible, or production proof.

## Local reproduction

Run `scripts/run-operations-lab.py --output /tmp/a-ops-evidence.json` from a
clean checkout. It performs two clean exports and two no-cache full Compose
runs, retains their separate outcomes, and cleans each project. A cached local
`target/`, Gradle home, Maven home, Docker build layer, or generated file must
not be required for success.

The local lab and hosted workflow both create source closure through
`scripts/create-source-git-bundle.sh`. A shallow checkout, missing parent,
additional or renamed advertised ref, wrong commit, incomplete import, or
exported tracked-byte difference fails before release construction.
