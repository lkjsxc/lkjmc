# Menu documents

## Purpose

`contracts/menus/*.json` is a repository-only static catalog for reviewing the
withdrawn JVM menu shape and generating route-reference Markdown. It is not
bundled into `/lkjmc-docs-bundle.json`, packaged as a JVM resource, or read by
`LocalDocsMenu`.

## Current catalog

The catalog retains `docs-directory`, `docs-file`, `docs-links`, and
`docs-search` as documentation metadata only. The local Paper `/menu`, `/docs`,
and hotbar entrypoints read the bundled documentation bundle directly; their
runtime behavior does not depend on these JSON files.

Daemon-backed root, server, admin, travel, claim, economy, social, profile,
settings, and adventure menus are withdrawn pending trusted adapter identity
and session attestation. Do not add placeholder documents for them.

## Validation boundary

`check-menus.py` applies hand-coded checks to the catalog's JSON decoding, ids,
kinds, themes, sizes, local bindings, title locale keys, parent links,
reachability, and generated route-document parity. It rejects daemon-shaped data and static
slots in the retained local-only catalog.

It does not prove Java packaging, Java consumption, slot rendering, action
execution, list grammar, or locale rendering. `check-jvm-containment.py`
separately rejects withdrawn Java surfaces in sources, test resources,
production resources, metadata, and built jars. Platform tests cover the
retained local Paper and Velocity registrations.

## Change procedure

1. Edit a repository catalog document only when its review metadata changes.
2. Update English and Japanese title keys when metadata introduces one.
3. Run `scripts/check-menus.py` and `scripts/generate-menu-docs.py --check`.
4. Regenerate the route-reference Markdown with
   `scripts/generate-menu-docs.py` when the catalog changes.

A catalog change alone never creates a Java route or capability.
