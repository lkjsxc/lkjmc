# Player profile

## Purpose

This document defines target profile data captured by plugin-enabled servers.

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

Inventory snapshots are immutable byte payloads with searchable JSON metadata.
Current state points to a snapshot revision.
