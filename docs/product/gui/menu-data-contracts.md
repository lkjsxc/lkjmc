# Menu data contracts

## Purpose

This file defines typed data admitted to menu rendering.

## Status

planned

## Sources

Paper consumes only the generated menu bundle, curated bundled documentation,
and immutable A-JVM sync snapshots. Menu, permission, claim, and settings
snapshots are integrated directly; profile, routing, and presence remain typed
where a route declares them. Generic JSON does not cross the loader or runtime.

Each dependency has domain, scoped key, revision, freshness state, and typed
payload. Current data may render rows. Stale data renders a labelled warning and
no mutation. Unavailable data renders a localized unavailable row. A missing or
wrong payload never becomes an empty successful list.

## Authority

Permission snapshots are hints only when current and exact for the player.
Mutation also requires trusted attestation and its named capability. A platform
permission, cached stale grant, `op`, route visibility, or rendered button is
not authority. This task introduces no daemon mutation transport.

## Local docs

`contracts/docs-player-corpus.json` is the complete player-visible corpus. Docs
lookups normalize paths and cannot read the host filesystem.
