# Docker release recovery lab

## Purpose and evidence ceiling

`scripts/run-docker-release-recovery-lab.py` is the sole operator entrypoint for a disposable,
test-owned Docker acceptance lab. It consumes exact retained release artifacts and immutable
PaperMC server assets, constructs a canonical-path host with real systemd and PostgreSQL, and drives
the packaged updater and recovery commands. The fixture code is not included in the fourteen-file
product release and is not a clean installer or runtime dependency.

The lab may establish disposable Docker installation, process, PostgreSQL, private-network,
operator, and Minecraft status-protocol evidence. It does not establish Docker production support,
a fresh supported-host installation, Incus/LXD isolation or restart, a public exposure boundary, a
real-player login, or production behavior.

## Safety and prerequisites

The driver accepts only a local Unix or loopback Docker context with cgroup v2 and the systemd
cgroup driver. The current rootful substrate is deliberately non-privileged: it drops all
capabilities, restores a fixed twelve-capability closure, uses a private cgroup namespace and
internal Docker network, and publishes no port. `SYS_ADMIN` and `apparmor=unconfined` are limited to
the test-owned systemd host so its entrypoint can mount cgroup v2 inside a private tmpfs cgroup
namespace. It retains Docker's default seccomp profile. Host network/PID namespaces, host or named
volumes, devices, the Docker socket, and `privileged: true` are rejected before boot.

The full matrix requires at least 10 GiB of Docker engine memory, eight CPUs, 5 GiB of available
workspace storage, and 30 GiB available on the filesystem that actually backs Docker's reported
data root. The smaller substrate probe requires 1 GiB on that Docker filesystem. The driver measures
both filesystems independently; free space beside the checkout cannot satisfy Docker-layer capacity.
Mutable scenarios are serialized. Every object has a unique
`io.lkjmc.docker-release-recovery.project` label; cleanup inspects that exact label and refuses an
unowned object.

Minecraft startup additionally requires explicit operator-controlled EULA acceptance. Docker
availability, repository text, a historical server, and a prior agent statement are not consent.
Without the explicit value the fixture returns `BLOCKED` before creating a service identity,
PostgreSQL cluster, canonical product root, EULA marker, or Minecraft process.

## Prepare exact inputs

Create one new private parent and name the exact successful `main` workflow artifact IDs and commits.
The target must be the retained artifact for the final implementation commit.

```sh
LKJMC_DRR_PRIVATE=$(mktemp -d)
chmod 0700 "$LKJMC_DRR_PRIVATE"
./scripts/run-docker-release-recovery-lab.py prepare-inputs \
  --baseline-artifact-id 9725523129 \
  --baseline-commit 23ad8d8ef389a453f71ffb3b0a7e333ea1e4a9d4 \
  --target-artifact-id "$TARGET_ARTIFACT_ID" \
  --target-commit "$TARGET_COMMIT" \
  --input-root "$LKJMC_DRR_PRIVATE/inputs" \
  --output "$LKJMC_DRR_PRIVATE/input-preparation.json"
```

`prepare-inputs` verifies artifact-service metadata, the successful required jobs, raw outer ZIP
digest, exact three-file handoff, canonical archive, manifest, fourteen artifacts, modes, and
embedded identities. It resolves Folia `1.21.11` and Velocity `3.4.0-SNAPSHOT` through PaperMC's
maintained API, chooses the highest stable build, downloads the content-addressed object, and freezes
its URL, build, size, and SHA-256. Runtime startup performs no catalogue or dependency resolution.

The resulting `lab-input-v1.json` also binds the pinned Ubuntu manifest, dated package snapshot,
exact package lock, Java/PostgreSQL majors, Compose definition, and every runtime-image build input.
Inputs, release bytes, server jars, and consumer receipts remain outside Git with private modes.

Only an operator who has explicitly accepted the Minecraft EULA may add
`--accept-minecraft-eula` to `prepare-inputs` or the later `full-matrix` invocation. The result records
only that the gate was supplied, not private consent material.

## Run the matrix

First validate the descriptor. This is read-only and reports absent target or consent as `BLOCKED`.

```sh
./scripts/run-docker-release-recovery-lab.py input-check \
  --input-descriptor "$LKJMC_DRR_PRIVATE/inputs/lab-input-v1.json" \
  --output "$LKJMC_DRR_PRIVATE/input-check.json"
```

After the target and consent gates are satisfied, run the complete matrix:

```sh
./scripts/run-docker-release-recovery-lab.py full-matrix \
  --input-descriptor "$LKJMC_DRR_PRIVATE/inputs/lab-input-v1.json" \
  --output "$LKJMC_DRR_PRIVATE/full-matrix.json"
```

When acceptance is supplied at execution rather than descriptor preparation, add
`--accept-minecraft-eula` to that command. The full path performs, in dependency order:

1. a fresh baseline install through the packaged artifact installer and packaged systemd unit;
2. a stopped-container Docker rollback image with an exact descriptor;
3. wrong-manifest no-effect preflight, changed packaged update, independent updater-backup
   verification, exact no-op fingerprints, service restart, and Docker restart;
4. exact backup handoff to a separate low-resource systemd/PostgreSQL host, packaged restore into an
   empty database, target daemon boot, direct retained-data queries, and proof that the updated source
   remained running and unchanged;
5. a fresh baseline update externally frozen and killed only after the regular fence and `prepared`
   journal coexist with the unchanged migration marker, followed by blocked ordinary/container
   starts and packaged recovery to the baseline;
6. the same fresh interrupted boundary with a disposable changed migration marker, proving packaged
   recovery refuses binary rollback and retains restore-required state.

A missed interruption window invalidates that fresh project. The driver may retry it in at most two
additional fresh projects; it never patches the updater, systemd, or product validation to widen the
race.

## Evidence, diagnosis, and cleanup

Every command has a deadline and bounded output. The retained result contains exact release, image,
asset, Docker, systemd, process, PostgreSQL, listener, backup, fence, journal, fingerprint, transition,
and cleanup identities without raw secrets. The sibling `.index.json` binds the result's size, mode,
and SHA-256 and explicitly excludes itself to avoid recursive hashing. The repository's canonical
secret scanner must accept both files before publication. Dumps, server jars, worlds, raw secrets,
and unbounded logs are never included.

Success and failure both run exact-label cleanup for containers, networks, volumes, runtime images,
rollback images, and operation-owned private handoff files. Behavioral success plus cleanup failure
is `FAILED`. If a project remains, use its exact identity from the private result to diagnose it; do
not delete by a broad `lkjmc` prefix or touch an unlabeled object.

`preflight`, `systemd-probe`, and `fixture-consent-gate` are bounded diagnostic modes. They do not
promote a missing full-matrix row to pass.
