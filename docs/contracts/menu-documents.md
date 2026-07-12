# Menu documents

## Purpose

`contracts/menus/*.json` is a repository-only static catalog for reviewing the
withdrawn JVM menu shape and generating route-reference Markdown. It is not
bundled, packaged as a JVM resource, or read by `LocalDocsMenu`.

## Current catalog

The catalog retains `docs-directory`, `docs-file`, `docs-links`, and
`docs-search` as documentation metadata only. The local Paper `/menu`, `/docs`,
and hotbar entrypoints read the bundled documentation bundle directly; runtime
behavior does not depend on these JSON files.

Daemon-backed root, server, admin, travel, claim, economy, social, profile,
settings, and adventure menus are withdrawn pending trusted adapter identity
and session attestation. Do not add placeholder documents for them.

## Validation boundary

`check-menus.py` validates every document named by `README.json` and rejects an
unindexed JSON document. It checks exact JSON members, local bindings, locale
titles, parents, reachability, and generated route-reference parity. It rejects
daemon-shaped data and static slots in the retained catalog.

It does not prove Java packaging, slot rendering, action execution, list grammar,
or locale rendering. `check-jvm-containment.py` separately rejects withdrawn
Java surfaces in sources, resources, metadata, and built jars.

## Change procedure

1. Edit a catalog document only when its review metadata changes.
2. Update English and Japanese title keys when metadata introduces one.
3. Run `scripts/check-menus.py` and `scripts/generate-menu-docs.py --check`.
4. Regenerate route-reference Markdown when the catalog changes.

A catalog change alone never creates a Java route or capability.
