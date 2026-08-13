# lkjmc

`lkjmc` is being recovered into a small single-host Minecraft network control
plane. The intended first supported topology is one private Rust daemon and
PostgreSQL database, one operator CLI, one Velocity proxy, and two private
Paper/Folia backends (`hub` and `survival`).

## Current state

The inspected source builds and its Rust and JVM unit suites pass, but it is not
yet a releasable product:

- the current command registry exposes far more operations than have real
  effects;
- Velocity does not register `/lkjmc`;
- the Paper/Folia menu advertises broad routes whose mutations are unavailable;
- the existing home-server deployment has Velocity and one hub, but no survival
  backend and no current player-path evidence;
- installation still depends on historical layouts rather than one verified
  immutable release.

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

Release installation and production operations instructions will be published
only after they have run from an exact verified release artifact. The current
historical checkout-based installer is not a supported production quick start.

## Version and license

Rust and JVM components share the pre-release version `0.1.0-alpha.1`. The
repository and package metadata use Apache-2.0. `lkjmc version` reports the
embedded version and source identity; this is identification, not evidence that
a release or player journey passed.
