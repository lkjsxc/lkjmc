# Contract checks

## Purpose

This document defines static repository checks and the boundary between those
checks, build verification, and opt-in live smoke evidence.

## Source map

| Boundary | Source |
| --- | --- |
| static documentation and line checks | `scripts/check-docs.py`, `scripts/check-doc-coverage.py`, `scripts/check-lines.py` |
| fast tier | `scripts/verify-fast.sh` |
| full tier and Gradle output | `scripts/verify-full.sh` |
| live guard selection | `scripts/verify-live.sh` |
| Compose full tier | `docker-compose.yml` `verify` service |

## Current static checks

| Check | Coverage |
|---|---|
| `scripts/check-command-docs.py` | Daemon command literals, CLI families, Paper command metadata, Paper permissions, Velocity root registrations, and `/lkjmc` docs. |
| `scripts/check-permissions.py` | `PermissionNodes.java`, Paper `plugin.yml`, and permission owner docs. |
| `scripts/check-locales.py` | English and Japanese catalog leaf keys in repository config and JVM resources. |
| `scripts/check-docs.py` | README tables of contents, links, H1s, purpose headings, statuses, stale state-source paths, and banned release-label terms. |
| `scripts/check-doc-coverage.py` | Tracked Markdown coverage tree, hashes, repository-contained evidence paths, actions, review commits, and implemented state-matrix source plus deterministic proof grammar. |
| `scripts/check-lines.py` | Recognized text files outside its explicit skip rules; it is not limited to Git-tracked files. |
| `scripts/check-menus.py` | Menu JSON structure, references, reachability, and generated route-doc parity. |
| `scripts/generate-menu-docs.py --check` | Generated route tables match `contracts/menus/*.json`. |
| `scripts/check-bootstrap-docs.py` | Bootstrap command docs, EULA guidance, ports, forwarding, and fallback `hub`. |
| `scripts/check-asset-docs.py` | Plugin IDs, hashes, Via dependency, and Geyser/Floodgate key handling. |

## Verification boundary

Fast runs the static checks plus Rust format, clippy, and workspace tests. Full
runs fast scope plus adapter scripts and `./gradlew --no-daemon test shadowJar`.
The default wrapper executes full. Compose's `verify` profile supplies the
PostgreSQL environment for full verification. Live is separate and runs a smoke
only when its guard is `1`; a skipped smoke is not a pass.

## Line and generated-output boundary

The line checker skips generated Gradle output only below
`platforms/jvm/**/build/**`. It checks an authored path such as
`platforms/authored/build/` even though its name resembles output. Its safety
probe creates both 201-line adversarial paths and requires only the JVM generated
path to skip. The checker must not create product state or print secrets.

Implemented state proof code spans must name an existing regular repository
file, `cargo test -p <workspace-package>`, or the configured verify Compose
command. The checker validates the file, Cargo package, or Compose service;
URLs and arbitrary text are not deterministic proof.
