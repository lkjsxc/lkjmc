# Contract checks

## Purpose

This document defines deterministic repository checks that keep docs, source,
and generated metadata aligned.

## Current checks

| Check | Coverage |
|---|---|
| `scripts/check-command-docs.py` | Daemon command literals, CLI families, Paper command metadata, Paper permissions, Velocity root command registrations, and `/lkjmc` docs. |
| `scripts/check-permissions.py` | `PermissionNodes.java`, Paper `plugin.yml`, and permission owner docs. |
| `scripts/check-locales.py` | English and Japanese catalog leaf keys in repository config and JVM resources. |
| `scripts/check-docs.py` | README tables of contents, links, H1s, purpose headings, statuses, and banned release-label terms. |
| `scripts/check-lines.py` | The 200-line file limit for tracked text files. |

## Menu document checks

| Check | Coverage |
|---|---|
| `scripts/check-menus.py` | Menu JSON parsing, ids, kinds, slots, chrome collisions, regions, confirmation reasons, locale keys, daemon command targets, route params, reachability, and generated route-doc parity. |
| `scripts/generate-menu-docs.py --check` | Generated route tables under `docs/product/gui/routes/` match `contracts/menus/*.json`. |

## Playable checks

| Check | Coverage |
|---|---|
| `scripts/check-bootstrap-docs.py` | Bootstrap command docs, quickstart EULA acceptance, playable Java and Bedrock ports, Velocity modern forwarding, and fallback `hub`. |
| `scripts/check-asset-docs.py` | Known plugin IDs, hash verification, ViaBackwards dependency on ViaVersion, and Geyser/Floodgate key handling. |

## Scope boundaries

Contract checks are deterministic. Parser, permission, completion, menu
contract, inventory lifecycle, and token-material tests belong in JVM unit
checks. Live downloads, Docker, and Minecraft server launches belong in opt-in
smoke checks unless a stable local cache makes them repeatable.

## Rule

A check may fail on drift, but it must not create product state or print
secrets.
