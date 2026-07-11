# Player profile

## Purpose

This document defines target profile data captured by plugin-enabled servers.


## Status

implemented

## Scopes

- Inventory contents
- Armor contents
- Offhand item
- Selected hotbar slot
- Ender chest
- Experience and level
- Health, food, saturation, air, and safe potion effects
- Game mode when enabled by policy
- `lkjmc` persistent plugin data
- Homes, warps, points, achievements, settings, and language

## Storage rule

Inventory snapshots are immutable, versioned JSON payloads with bounded typed
item bytes encoded as Base64 and searchable JSON metadata. The adapter validates
shape, bounds, and numeric fields before applying it; it never uses Java object
deserialization. Current state points to a snapshot revision.

## Current status

The current slice implements PostgreSQL lease helpers, immutable snapshot writes,
daemon `player.inspect`, `player.load`, `player.snapshot`, and
`player.restore` commands, and CLI inspect/snapshot/restore commands. Paper
join-time load, session records, save-on-quit, transfer acknowledgement, and
proxy wait-for-save are withdrawn pending trusted identity/session attestation.
`player.recovery.report` records an audit-backed report for operator review and
does not perform an automatic repair action.
