# Menu documents

## Purpose

`contracts/menus/*.json` defines bundled local documentation-menu routes. One
route file defines one route, and the filename equals the route id.

## File format

A shipped menu document has a local docs binding or no data source. It defines
an id, kind, title locale key, docs theme, supported inventory size, parent hint,
chrome, list grammar, static slots, and no confirmation reason.

## Local action grammar

- `open`, `back`, and `close` navigate local route history.
- `input` accepts a bounded local documentation-search line.
- `message` presents a documented external link.
- `none` is inert and is required for decoration and information roles.

No shipped document may contain a player command, daemon command, transfer,
mutation, daemon source, refresh, grant condition, or confirmation action.

## Shipped routes

The docs directory, file, links, and search routes read the bundled docs bundle.
They may paginate and preserve route history. The hotbar token and `/menu` enter
the same local docs route; `/docs` can select a path or search.

## Withdrawn routes

Root, server, admin, travel, claim, economy, social, profile, settings, and
adventure documents are withdrawn pending trusted identity/session attestation.
Do not add a placeholder document for a daemon capability.

## Validation

`check-menus.py` verifies the exact local route catalog, JSON decoding, ids,
kinds, themes, sizes, local data bindings, title locale keys, parent links,
parent-chain reachability, and generated route-doc parity. It rejects static
slots and daemon-shaped data because no shipped local document uses either.
The containment checker rejects withdrawn Java surfaces in source, test
resources, production resources, metadata, and built jars.

## Change procedure

1. Edit or add a bundled local docs document.
2. Update English and Japanese locale keys.
3. Run `scripts/check-menus.py`.
4. Regenerate route docs and the JVM resource index with
   `scripts/generate-menu-docs.py`, then commit the generated route catalog.

Do not add fake menu documents or Java daemon behavior.