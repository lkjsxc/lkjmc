# lkjmc

`lkjmc` is being recovered into a small single-host Minecraft network control
plane. The intended first supported topology is one private Rust daemon and
PostgreSQL database, one operator CLI, one Velocity proxy, and two private
Paper/Folia backends (`hub` and `survival`).

## Current state

The exact `e87c3237db0e256fb4af225c93ab6e4fd4660a67` release currently serves one
Velocity proxy plus private hub and survival Folia backends from an unprivileged
Incus container. Clean install, restart recovery, public status ping, private
port boundaries, backup, and restore have been observed. The product is not yet
player-accepted:

- the current command registry exposes far more operations than have real
  effects;
- source after the serving release adds the small `/lkjmc` command, but it still
  needs an exact release deployment and real-client command/completion/transfer
  evidence;
- the Paper/Folia menu advertises broad routes whose mutations are unavailable;
- plugin heartbeat is absent, so daemon status truthfully reports backend
  readiness as unknown; and
- there is no supported unattended immutable installer yet.

Do not infer support from dormant handlers, generated contracts, menu routes,
Compose services, or guarded checks that did not run. Current implementation and
live evidence are recorded in [the active work ledger](docs/work/active.md).

## Recovery scope

The first release will support only:

- daemon and database health;
- desired versus observed instance status;
- start, stop, restart, logs, reconcile, backup, and restore through the CLI;
- a real `/lkjmc status` and `/lkjmc server <id>` path on Velocity;
- transfer between `hub` and `survival`;
- a small backend menu for status, server selection, and language;
- release installation into one unprivileged Incus/LXC system container.

Economy, claims, adventures, mail, Discord, Kubernetes, Bedrock, public web
administration, and other broad historical domains are deferred and will be
removed from the default product surface.

## Development baseline

Prerequisites currently exercised are Rust 1.97, Java 21, and Python 3.12.
PostgreSQL is required for the integration tier.

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./gradlew --no-daemon --no-build-cache test
```

These are deterministic source checks only. They do not prove a real Minecraft
network, deployment, login, command, menu click, transfer, backup, or restore.

## Agent and operator entry points

- [Repository operating contract](AGENTS.md)
- [Current objective, evidence, blockers, and next command](docs/work/active.md)

The active ledger records the exact private deployment and rollback evidence.
The inherited checkout-based installer is not a supported production quick
start; it has not yet been replaced by the immutable procedure that was used for
the serving container.

## Version and license

Rust and JVM components share the pre-release version `0.1.0-alpha.1`. The
repository and package metadata use Apache-2.0. `lkjmc version` reports the
embedded version and source identity; this is identification, not evidence that
a release or player journey passed.
