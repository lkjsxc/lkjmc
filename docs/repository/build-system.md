# Build system

## Purpose

This document defines repository build ownership and its output boundary.

## Source map

| Concern | Source |
| --- | --- |
| Rust workspace and gates | `Cargo.toml`, `scripts/verify-fast.sh` |
| Java 21 and `shadowJar` task | `build.gradle.kts`, `platforms/jvm/*/build.gradle.kts` |
| container stages | `Dockerfile` |
| Compose profiles and verification service | `docker-compose.yml` |

## Rust

The Rust workspace owns core models, store helpers, daemon adapters, CLI
parsing, and local runtime orchestration. The fast and full scripts run `cargo
fmt --check`, workspace clippy, and workspace tests. The full script adds
adapter checks; Compose supplies PostgreSQL for database-backed tests.

## Java

Gradle builds Java 21 platform plugins and common JVM contracts. `shadowJar`
creates archives under module `build/` directories; these are real build outputs
used by verification and asset workflows, not tracked source artifacts.

The common module compiles Adventure API and MiniMessage as `compileOnly` and
tests them as `testImplementation`. Paper API `1.21.10-R0.1-SNAPSHOT` resolves
Adventure `4.25.0`; Velocity API `3.4.0-SNAPSHOT` resolves Adventure `4.26.1`.
Common pins `4.25.0`, the lower platform-provided API, so Paper remains
compatible while Velocity supplies a newer runtime. Paper tests load the local
document bundle and plugin metadata and assert the slot-8 token constant;
Velocity tests cover MOTD fallback and tab-list text. They do not start Paper or
a Velocity proxy.

## Docker

Compose defines PostgreSQL plus `verify`, `playable`, and `discord` profiles.
The `verify` service runs `./scripts/verify-full.sh`; `playable` owns named
runtime volumes. Those volumes and all container outputs are runtime state, not
repository artifacts.

## Release naming

Docs and generated names must not use artificial product release labels. Use
`dev`, commit identifiers, or content hashes when a machine field needs a value.
