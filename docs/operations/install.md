# Immutable release update

## Supported scope

The supported entrypoint updates an **existing healthy system deployment** of
the fixed single-host topology. It does not provision a host, PostgreSQL,
server assets, secrets, credentials, Incus networking, or a Minecraft EULA
acceptance record. Clean installation is still blocked and `scripts/install.sh`
exits without mutation rather than building checkout bytes or overwriting
operator intent.

A release can be built in a detached clean worktree for local verification:

```sh
scripts/build-release.sh "$HOME/lkjmc-private-releases/$COMMIT"
```

Retain the printed release root and the SHA-256 of
`artifact-manifest.json` through a separate operator channel for that local
result.

The required `main` workflow also retains the already compared release bytes
for 30 days. Its canonical artifact name is
`lkjmc-release-$COMMIT-run-$RUN_ID-attempt-$ATTEMPT`; it is not `latest` and is
not a GitHub Release. The outer artifact contains only the canonical tar,
archive sidecar, and `release-handoff.json`. Obtain the exact run, artifact ID,
artifact-service digest, and expiry through the GitHub Actions API. Download
the raw outer ZIP and recompute that service digest, then download its files
into a new private operator directory. Normalize only the outer transport
directory through `scripts/private-artifact-handoff.py`; outer modes are not
installed authority.

From a clean checkout of the exact workflow commit on the operator machine,
use `scripts/release_archive.py verify` with the repository, artifact name,
event, ref, run, attempt, and producer job recorded by the descriptor and
workflow. Use `consume` for read-only verification plus automatic temporary
cleanup, or `extract` with a nonexisting private output to retain the verified
release root. Both commands reject a missing or extra outer file, wrong
workflow fact, digest or sidecar disagreement, malformed USTAR metadata,
unsafe member, closure or mode difference, and extraction conflict. `consume`
runs the existing manifest and embedded-identity verifiers without compiling
Rust, running Gradle, resolving release dependencies, or rebuilding bytes.
Do not substitute `tar -xf`, an unpacked artifact upload, or a manifest-only
download for this path.

The extracted `artifact-manifest.json` SHA-256 reported through the separately
verified handoff remains the updater's external anchor. Transfer the extracted
release root and that digest to the existing deployment; the production
runtime needs neither a source checkout nor a build toolchain. Artifact
download or extraction does not authorize or prove an update. The update
preflight, backup, fence, restart, no-op, restore, and recovery boundaries below
remain separate live operator actions.

After private transfer, run the deployer that is itself inside that anchored
release:

```sh
sudo "$RELEASE/source/lkjmc-deploy-release" update \
  --release-root "$RELEASE" \
  --manifest-sha256 "$MANIFEST_SHA256" \
  --from-commit "$CURRENT_COMMIT" \
  --backup "/var/backups/lkjmc/pre-$COMMIT/lkjmc.dump" \
  --rollback-snapshot "$INCUS_SNAPSHOT"
```

The snapshot is an explicit host-operator rollback assertion. A process inside
the container cannot verify the Incus host snapshot, so the command records its
label but does not claim it observed the snapshot.

## Update contract

Before stopping anything, `lkjmc-deploy-release` requires:

- root, systemd, the existing `lkjmc` service account, local PostgreSQL, and the
  canonical fixed paths. Required commands are fixed by absolute name; each
  root-owned symlink in a command chain and its non-writable ancestry is checked,
  and the final executable must remain below that command's explicit system root.
  On supported Ubuntu, the PostgreSQL commands may resolve only within
  `/usr/bin` or `/usr/share/postgresql-common`, which admits the packaged
  `pg_wrapper` without admitting arbitrary root-owned command targets;
- a root-owned current release whose manifest, binaries, jars, and embedded
  commit agree with `--from-commit`;
- the externally supplied manifest digest and exact fourteen-file product,
  operations-tool, restart-helper, privileged fence-checker, service-unit, and
  deployment-fence release closure, all under a root-owned non-writable path;
- an active daemon, connected database, no-op bootstrap plan, exact
  `proxy`/`hub`/`survival` topology, immutable server-asset hashes, fresh
  backends, proxy registrations, plugin heartbeats, and exact installed plugin
  jars;
- the three existing instance-bound heartbeat credential files with private
  ownership and modes;
- an existing EULA record: either the root-managed versioned marker or both
  existing backend `eula.txt` files containing `eula=true`.

The deployer never creates that acceptance record. It refuses an existing
backup destination, creates a fresh private PostgreSQL custom dump under
`/var/backups/lkjmc`, independently runs `pg_restore --list` and schema
extraction, and requires its checksums, schema hash, migration marker, and
source commit to match the live pre-update state.

One root-owned global lock serializes no-op, update, and recovery. Before any
runtime publication, the updater installs an effective systemd fence drop-in,
writes a durable root-owned fence and recovery journal, then stops the complete
systemd cgroup. A reboot or crash cannot normally start either release while
the fence remains. One root-owned `/run` permit allows only the updater's
fenced verification start: a privileged fail-closed `ExecStartPre` validates
the control-file ancestry and atomically consumes the permit before daemon
execution. Any automatic retry while the fence remains has no permit and is
blocked. With the service user absent, plugin files are published through
no-follow directory descriptors; the unit and current pointer are then switched
and migrations run while stopped.

Systemd startup invokes the packaged restart helper, which waits for the daemon
socket and performs the one supported playable bootstrap reconciliation.
Success requires a real local proxy status ping, a no-op post-start plan, exact
new identities, two ready/joinable backends, fresh registration/heartbeat
state, and `NRestarts=0`. The permit and fence are removed only after those
checks pass.

An identical release is a verified no-op: it does not back up, migrate, switch
pointers, replace files, or restart processes.

## Failure and rollback

Artifact publication uses `lkjmc-install-artifacts`: it verifies the anchored
manifest, stages on the target filesystem, fsyncs files and directories, and
renames atomically. A failure before publication commit restores the exact prior
artifact tree. A cleanup error after a validated durable commit retains the new
tree instead of deleting both valid trees. Versioned deployment targets are
never replaced when an existing target differs.

The deployment coordinator records the migration ledger before and after the
attempt:

- before any ledger change, a failure restores the previous unit, current
  pointer, and plugin bytes, then starts and verifies the previous release;
- after a changed or unreadable migration ledger, binary-only rollback is
  forbidden. The service is stopped and the receipt names the database backup
  and Incus snapshot required for a data-aware restore.

After an interruption with a retained fence, invoke the exact deployer from the
same anchored target release:

```sh
sudo "$RELEASE/source/lkjmc-deploy-release" recover \
  --release-root "$RELEASE" \
  --manifest-sha256 "$MANIFEST_SHA256" \
  --to-commit "$TARGET_COMMIT"
```

Recovery stops any process first. It restores and verifies the old unit,
pointer, and plugin bytes only when the live migration ledger is exactly the
recorded pre-update ledger. A changed or unreadable ledger keeps the service
fenced and reports the required backup and Incus snapshot.

For migration 53 specifically, an older binary is not compatible with the new
ledger. Restore the matching pre-migration database/snapshot together with the
matching old release; do not only repoint `current`.

## Boundaries

The updater does not alter the Incus proxy device, firewall, DNS, container
privilege, public ports, or server-jar acquisition. Only Velocity TCP `25591`
may be exposed; daemon HTTP, PostgreSQL, and backend listeners remain private.
Update success is not a real-player login, command, completion, transfer, or
menu observation.

## Verification

```sh
./scripts/check-installer.sh
LKJMC_SECRET_CANARY="$(openssl rand -hex 24)" \
  ./scripts/operations-artifact-install-drill.sh "$PRIVATE_EVIDENCE/artifacts"
```

These deterministic drills prove anchored publication, stable artifact no-op,
pre-commit publication rollback, fake-backup rejection, serialization, EULA
parsing, and the migration rollback classification. A supported release
additionally requires a real PostgreSQL backup/restore and update/no-op/restart
drill in a disposable unprivileged systemd LXC plus exact production
observations; tests do not substitute for that live boundary.
