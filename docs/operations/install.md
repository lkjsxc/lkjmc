# Immutable update and recovery

## Supported scope

`lkjmc-ops deploy update` updates an existing systemd-supervised lkjmc deployment. `lkjmc-ops host
install` is the separate UUID-bound first-install operation for a prepared systemd container. Clean
installation remains unsupported until its fresh-supported-host evidence is independently observed.
It does not create a container, install operating-system packages, configure public networking, or
become a general host provisioner. See the
[host deployment entrypoint](quickstart/host-install.md) for its input and acceptance contract. A
downloaded or extracted release is not installed or running evidence.

The operation accepts a bounded typed fleet with one configured Velocity entrypoint and arbitrary
valid instance IDs. The backend count is not fixed. Paper, Folia, and Purpur use plugin-heartbeat
readiness; an active custom/modded kind with unsupported readiness fails preflight. Intentionally
stopped instances remain stopped.

## Host EULA policy

For an operator-owned lkjmc environment, create the single auditable policy once:

```sh
sudo /opt/lkjmc/releases/current/bin/lkjmc-ops eula policy create \
  --config /etc/lkjmc/lkjmc.json
```

The command derives the unprivileged service identity from the validated managed-instance root and
atomically writes the exact versioned policy as root with its private service group. It rereads and
hashes the result. This records only that operator's policy; it is not consent for another operator
or host.

Before every systemd start, the unit invokes `lkjmc-ops eula materialize`. It enumerates every
configured kind that requires Mojang's EULA, rejects symlink or ownership ambiguity, writes exact
`eula=true` state atomically, and returns an instance-indexed receipt. Instance creation has no EULA
boolean and does not imply startup. A missing policy leaves affected instances stopped.

## Read-only release verification

Use the exact `lkjmc-ops` inside the anchored target release:

```sh
sudo "$RELEASE/source/lkjmc-ops" release verify \
  --release-root "$RELEASE" \
  --manifest-sha256 "$MANIFEST_SHA256"
```

The manifest SHA-256 must arrive through an independent trusted channel. This Rust command checks
the strict sidecar, exact eight-member inventory, path, size, digest, and release-source mode. It
performs no installation. System install and update additionally require a root-owned private source
tree with trusted ancestry. The independent release builder and artifact consumer own ELF/JAR type,
embedded build-identity, archive-closure, and interpreter-absence checks; a successful Rust manifest
check alone does not promote those boundaries.

## Changed update

```sh
sudo "$RELEASE/source/lkjmc-ops" deploy update \
  --operation-id "$OPERATION_UUID" \
  --release-root "$RELEASE" \
  --manifest-sha256 "$MANIFEST_SHA256" \
  --from-commit "$CURRENT_COMMIT" \
  --from-manifest-sha256 "$CURRENT_MANIFEST_SHA256" \
  --config /etc/lkjmc/lkjmc.json \
  --backup "/var/backups/lkjmc/pre-$TARGET_COMMIT.dump" \
  --rollback-snapshot "$OPERATOR_SNAPSHOT_LABEL"
```

The snapshot label is an operator assertion recorded in the journal; a process inside the container
does not claim it observed the host snapshot.

For a source release installed by `host install`, the updater accepts only the root-owned
mode-`0640` systemd unit and fence drop-in that installer published, or the root-owned
mode-`0644` forms published by a prior changed update. Other ownership or mode drift is rejected.

Preflight verifies root privilege, trusted fixed tool paths, the running operations executable, both
release roots, typed configuration, service identity, PostgreSQL inventory equality, immutable
assets, inventory-derived plugins and credentials, the designated Velocity listener, EULA policy
and materialized files, current status, and supported readiness. It names the first divergent
instance and stops before service mutation.

One root-owned global lock serializes update and recovery. For a changed target, the operation:

1. fsyncs a preflight journal before EULA, rollback-input, or backup effects;
2. materializes and verifies inventory-derived EULA files and saves exact rollback inputs;
3. creates and independently verifies a new private PostgreSQL backup;
4. fsyncs the verified backup closure into the journal and writes the systemd fence;
5. stops the complete service through systemd;
6. stages and atomically publishes versioned artifacts, unit, drop-in, and inventory-derived jars;
7. applies ordered Rust-owned migrations while stopped and records ledger identity;
8. activates the target once and grants one matching, one-use start permit;
9. starts systemd; the service-user post-start sequence runs `lkjmc --json bootstrap apply` to
   reconcile the typed desired fleet, then runs `lkjmc-ops bootstrap after-start`;
10. compares build identity and every required dynamic fleet state; and
11. removes permit and fence only after acceptance.

No command is interpreted through a shell. Paths, ancestry, types, owners, modes, identities,
deadlines, and output bounds are checked before privileged effects.

## Exact no-op

A no-op requires the independently anchored target and complete current release identity to match,
with no stale fence or drift. It is read-only: no backup, migration, stop, start, pointer/plugin/unit
rewrite, credential change, EULA rewrite, or release rewrite occurs. Topology divergence or corrupt
bytes is an error, not a no-op.

## Failure and recovery

Before a migration-ledger change, a failed changed update restores and verifies the exact prior
unit, pointer, plugins, and release when safe. After a changed or unreadable ledger, binary-only
rollback is forbidden: the service remains fenced and the receipt names the matching release,
backup, and data-aware recovery requirement. The first causal error is retained even if cleanup also
fails.

Resume an interrupted operation only with the same operation UUID and exact packaged target binary:

```sh
sudo "$RELEASE/source/lkjmc-ops" deploy recover \
  --operation-id "$OPERATION_UUID" \
  --release-root "$RELEASE" \
  --manifest-sha256 "$MANIFEST_SHA256" \
  --config /etc/lkjmc/lkjmc.json
```

Recovery first validates the journal, release, fleet, database, backup, fence, and permit identities.
An interruption before the fence verifies that the source release is still running, removes only an
exact owned partial-backup stage, and records the operation as `abandoned` without stopping the
service. Once a matching fence exists, recovery stops the running service and either restores the
safe pre-ledger state or remains fenced with a precise restore blocker. Reusing the operation ID with
different inputs is rejected. Restart or reboot cannot bypass the durable fence, and a consumed start
permit cannot be replayed.

## Honest evidence boundary

Rust unit/process tests and exact release inspection prove deterministic behavior only. PostgreSQL
tests add real migration, inventory, backup, and isolated restore evidence. Neither proves a fresh
supported-host installation, systemd/Minecraft readiness, player connection, or production state.
The active ledger records each tier separately.
