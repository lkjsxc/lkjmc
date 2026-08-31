# lkjmc

`lkjmc` is a small Minecraft control plane for a bounded, operator-defined fleet. Its supported
architecture is one private Rust daemon and PostgreSQL database, one explicit Rust operator CLI,
one selected Velocity player entrypoint, and any finite set of configured Paper-, Folia-, or
Purpur-compatible backends. Instance IDs are opaque: example names do not define role, kind,
readiness, routing order, or lifecycle.

## Current boundary

The canonical typed JSON configuration owns instances, listeners, routes, immutable assets,
integrations, readiness contracts, and desired state. PostgreSQL owns durable fleet and operation
facts; systemd and direct protocol/plugin observations own runtime truth. Running custom or modded
servers without a supported readiness contract is rejected rather than described as joinable.

`lkjmc-ops` is the one packaged installation, update, and recovery authority. It verifies anchored
releases, performs UUID-bound first-install preflight and mutation inside a prepared container,
publishes exact artifacts, materializes authorized Minecraft EULA files, creates and verifies
PostgreSQL backups, enforces the durable deployment fence and one-use start permit, performs exact
no-op and changed updates, classifies rollback, resumes interrupted updates, and emits bounded
receipts. The immutable payload contains three Rust binaries, three Java jars, and two declarative
systemd files. It contains no Python or shell executable and the installed service invokes no
interpreter-owned operational path.

The root-owned host EULA policy is the sole consent authority. Creating an instance does not claim
acceptance or startup. Before a managed server kind starts, `lkjmc-ops eula materialize` validates
that policy and atomically verifies the instance-owned `eula.txt`; a missing or unsafe policy keeps
the instance stopped.

Historical deployments and recovery exercises are evidence only for their exact old releases.
Current deterministic, PostgreSQL, artifact, installation, player, and production evidence remain
separate in [the active work ledger](docs/work/active.md). A build or passing unit test is not a
supported-host installation, Minecraft login, transfer, or production observation.

The first-install command is implemented, but clean installation remains unsupported until fresh
unprivileged Incus/LXD supported-host acceptance is observed. It must not be represented as
supported-host, player, or production evidence until that independent boundary is complete.

## Operator surface

- `lkjmc` is the explicit operator client for desired state, status, logs, and diagnosis.
- `lkjmc-daemon` owns authorization, persistence, reconciliation, processes, and private APIs.
- `lkjmc-ops` owns packaged release/update/recovery, backup/restore verification, fences, EULA
  materialization, and post-start acceptance.
- Velocity owns authenticated player identity, `/lkjmc`, routing registrations, and actual
  connection requests. Command completion and status enumerate the configured backend IDs.
- Paper/Folia-compatible plugins own backend-local behavior and heartbeat readiness.

See [immutable update and recovery](docs/operations/install.md),
[backup and restore](docs/operations/backup-restore.md), and
[release integrity](docs/operations/release-integrity.md).

## Development baseline

The pinned verifier uses Rust 1.97, Java 21, Gradle 8.10.2, and PostgreSQL 14. Local focused checks
are:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./gradlew --no-daemon --no-build-cache test shadowJar
```

`scripts/verify-full.sh` is the repository-wide deterministic and PostgreSQL verifier. Release
construction runs only from an exact clean commit; see the operations documents before treating its
output as installable bytes.

## Repository entry points

- [Repository operating contract](AGENTS.md)
- [Current objective and evidence](docs/work/active.md)
- [Architecture](docs/architecture/README.md)
- [Operations](docs/operations/README.md)

Rust and JVM artifacts share version `0.1.0-alpha.1` and Apache-2.0 licensing. Version output proves
identity only, not deployment or readiness.
