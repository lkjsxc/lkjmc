# Contract checks

## Purpose

This document defines static repository checks and their evidence boundary.

## Source map

| Boundary | Source |
| --- | --- |
| static documentation and line checks | `scripts/check-docs.py`, `scripts/check-doc-coverage.py`, `scripts/check-lines.py` |
| domain contracts | `scripts/check-contracts.py` |
| fast tier | `scripts/verify-fast.sh` |
| full tier and Gradle output | `scripts/verify-full.sh` |
| live guard selection | `scripts/verify-live.sh` |
| Compose full tier | `docker-compose.yml` `verify` service |

## Current static checks

| Check | Coverage |
| --- | --- |
| `scripts/check-contracts.py` | 137 real daemon registrations, closed request and nested domain shapes, complete shard manifest/generated include parity, every literal CLI/web body, withdrawn adapter results, config ownership, menu documents, bounded shards, and generated output. |
| `scripts/check-command-docs.py` | Compatibility entrypoint for the domain-contract check. |
| `scripts/check-permissions.py` | Local-safe Paper metadata and permission owner docs. |
| `scripts/check-menus.py` | Exact local route shape, locale titles, parent reachability, and generated route-doc parity. |
| `scripts/check-docs.py` | Markdown topology, links, statuses, and stale source paths. |
| `scripts/check-doc-coverage.py` | Coverage records, hashes, evidence paths, and implemented state rows. |
| `scripts/check-lines.py` | Authored text line limits, including contracts, outside generated-output skips. |
| `scripts/check-jvm-containment.py` | Sources, resources, metadata, and built jars lack withdrawn daemon clients, adapters, commands, bridges, and credentials. |

## Verification boundary

Fast runs static checks plus Rust format, clippy, and tests. Full additionally
runs Gradle with daemon and build cache disabled, then checks every built jar.
Compose supplies PostgreSQL for DB-backed verification. Live is separate and
dispatches only supported guarded lanes; blocked Java adapter paths are never a
pass.

## Generated-output boundary

The line checker skips generated Gradle output only below
`platforms/jvm/**/build/**`. The truth probe repeats manifest, shard, and
literal-payload checks with negative mutations so removal or genericization
cannot be hidden by a self-consistent edited index. Static checks must not
create product state or print secrets.
