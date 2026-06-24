# Jar registry

## Purpose

This document defines the central server jar registry behavior.

## Storage

The daemon stores jar files under the configured jar root. Imported custom jars
are copied into a content-addressed custom path and never overwritten in place.
Each stored jar has a PostgreSQL `jar_assets` row with kind, project, channel,
name, path, SHA-256, size, source, and metadata.

## Current implementation

The runtime implements local fixture import, listing, inspection by kind or
project, and launch-time checksum verification. `jar.import` accepts a local
path, calculates SHA-256, copies the file into the configured jar root, records
an immutable asset, and refuses duplicate stored paths. `jar.list` returns
stored assets. `jar.inspect` returns the newest matching asset.

Instance launch may use a `jarAssetId` in the stored instance config. Before the
process starts, the daemon reads the asset row, hashes the on-disk jar, and
refuses launch if the checksum differs. The generated launch command is:

```text
java -Xmx{memoryMb}M -jar {asset.path} nogui
```

## Current boundaries

`jar.sync` uses the PaperMC downloads service for Paper, Folia, and Velocity.
It sends the configured `lkjmc` User-Agent, selects stable builds by default,
downloads the server jar, verifies SHA-256 and size, and records a jar asset.
No command may report a download success until a real asset row and verified
file exist.

`jar.prune --yes` removes jar assets that no instance references. It deletes
the on-disk file, removes the asset row, and writes audit events. Referenced
assets are never pruned.

## Current boundaries

Live Minecraft smoke downloads are a separate slice.
