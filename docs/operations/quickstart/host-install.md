# Host deployment entrypoint

## Status

`lkjmc-ops host install` is the only packaged first-install authority. It is implemented for a
prepared systemd container, but clean installation remains unsupported until fresh unprivileged
Incus/LXD supported-host acceptance is independently observed for this revision. It is therefore
not currently a supported-host, player, or production claim. It does not create containers, install
operating-system packages, change a host firewall, or provision an arbitrary machine.

Existing-system immutable update remains separately documented in
[immutable update and recovery](../install.md).

## Prepared-substrate contract

Before the command changes lkjmc-owned state, it requires root; systemd as PID 1; Java 21 or newer;
PostgreSQL 14 server, client, `pg_dump`, and `pg_restore`; fixed root-owned utilities; a local
PostgreSQL administrative socket; sufficient declared disk, memory, process, and file-descriptor
capacity; and free configured listeners. The container preparation boundary, package acquisition,
and immutable server-jar acquisition are operator/proof infrastructure, not lkjmc product behavior.

The command accepts only a root-owned `0600` JSON input and an independently supplied SHA-256 of
that exact input. The release root, configuration source, and each asset source must likewise have
trusted root-owned ancestry and immutable identity. A source checkout or build toolchain is not a
target runtime dependency.

## First-install input

Input schema version `1` has one semantic owner in `lkjmc-ops`. It binds a non-nil operation UUID,
the anchored release commit and manifest digest, a canonical configuration file identity, the full
server-asset closure, fixed system roots, the dedicated service identity, the local PostgreSQL
contract, and capacity minima. Unknown fields are rejected.

The JSON object has these top-level fields:

| Field | Meaning |
| --- | --- |
| `schemaVersion` | Literal `1`. |
| `operationId` | A non-nil UUID used for durable replay and recovery identity. |
| `release` | `root`, exact 40-character `commit`, and `manifestSha256`. |
| `configuration` | Root-owned source `path`, `sha256`, and byte `size`. |
| `assets` | Unique server asset records: `id`, `kind`, `version`, `sourceIdentity`, and source identity. |
| `roots` | The packaged `/opt/lkjmc`, `/etc/lkjmc`, `/var/lib/lkjmc`, `/var/log/lkjmc`, `/var/backups/lkjmc`, and `/opt/lkjmc/runtime-assets` contract. |
| `service` | Dedicated `lkjmc` user/group, nonzero UID/GID, `/var/lib/lkjmc` home, and `/usr/sbin/nologin` shell. |
| `postgres` | Local `postgres` socket administration plus the new least-privileged role and database names. |
| `capacity` | Minimum free MiB, available MiB, processes, and open files. |

The canonical configuration is still the owner of instance IDs, kinds, listeners, routes, desired
states, integrations, readiness, and selected Velocity entrypoint. The install input cannot create
a second topology model. Every desired-running instance must name exactly one immutable required
server asset. Backend listeners must remain loopback; the typed Velocity listener is the only
possible player-facing listener.

For a first-install input, configure `assets.root` as `/opt/lkjmc/runtime-assets`, use only exact
server-asset paths directly below that root, and set `network.capabilities.mountedAssets` to `true`.
The illustrative names in repository examples are not reserved; the input itself must use actual
non-placeholder SHA-256 values, byte sizes, source identities, and a release manifest digest.

## Invocation

After extracting and independently verifying the exact release into a root-owned private release
root, invoke the operations binary from that same release:

```sh
sudo "$RELEASE/source/lkjmc-ops" host install \
  --input "$INSTALL_INPUT" \
  --input-sha256 "$INSTALL_INPUT_SHA256"
```

The input SHA-256 is an independent anchor, not a value discovered by the command from the input
itself. On a fresh target, the command acquires the global deployment lock, fsyncs a secret-free
journal before its first lkjmc effect, creates only exact operation-owned state, initializes the
database and canonical fleet through the daemon boundary, materializes the root-owned EULA policy,
publishes the exact unit and release, starts systemd under a one-use permit, and accepts only after
the private daemon, PostgreSQL, full fleet, configured readiness, and selected Velocity status
protocol all agree.

Its receipt is JSON and contains only `schemaVersion`, `result`, `operationId`, release and input
digests, fleet revision, configured instance IDs, and selected Velocity ID. It contains no password,
token, database URL, or secret path.

## Target classification and recovery

Before mutation, a target is classified as fresh, the same resumable UUID/input closure, an exact
accepted target, or a conflict. An existing user, group, numeric identity, root, database, service,
listener, fence, permit, release, secret, or foreign journal is a conflict; it is never adopted or
deleted. A changed input under the same UUID is rejected.

An accepted exact replay returns `"result":"no-op"` after re-verification. It does not generate
credentials, change EULA state, migrate PostgreSQL, republish assets/releases/units, reload
systemd, or start, stop, restart, back up, or replace the journal. A resumable operation checks each
completed effect independently and retains the first causal failure. Unsafe partial state remains
fenced instead of being reported as accepted.

## Evidence boundary

This command's source and deterministic tests do not establish a fresh supported-host installation.
Until a named, project-owned, unprivileged Incus/LXD system container independently completes the
installation, exact replay, service restart, container restart, backup, isolated restore,
unprivileged operator observation, and Minecraft status-protocol checks, those evidence states
remain unobserved. It does not claim real-player login, transfer, public reachability, or production
deployment.
