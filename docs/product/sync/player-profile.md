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

A snapshot is exactly schema `lkjmc-profile-one`: one typed JSON envelope for
inventory and armor slots, offhand, selected slot, ender chest, experience,
vitals, safe potion effects, optional game mode, plugin key/value data, homes,
warps, points, achievements, settings, and language. Unknown fields, duplicate
keys, invalid namespaced identifiers, non-finite numbers, and values beyond the
core limits are rejected. The envelope is at most 1 MiB; inventory, armor, and
ender chest have 41, 4, and 27 unique slots; potion effects and plugin data have
at most 128 entries; achievements have at most 1,024 entries; and homes and
warps each have at most 128 entries. Namespaced identifier components are at
most 128 bytes, plugin values 8,192 bytes, item names and lore lines 1,024 bytes,
and privacy 64 bytes. Canonical serialization sorts all set-like collections,
uses Rust field order with no insignificant whitespace, and computes SHA-256
from those bytes. Callers supply neither a format nor an integrity value.

Migration `045` does not deserialize or convert pre-cutover rows. It moves them
to a read-inaccessible quarantine table with reason `untyped-profile`, drops
the old payload columns, and requires a trusted future adapter to resave a typed
snapshot. Quarantined data is never returned as a profile.

## Transaction and replay rule

Identity/session revision, lease fence, snapshot revision, snapshot row, and
change-feed row commit together. The writer supplies one correlation plus the
expected active session revision, lease fence, and prior snapshot revision. A
stale value or changed replay is denied; an exact replay returns the same
snapshot revision and integrity.

## Current status

The daemon/store implement typed operator snapshot/load/inspect data operations.
Read-only sync may transport and cache the latest typed envelope by player/scope
revision. Paper join/load, save-on-quit, receipt application, and proxy transfer
remain withdrawn pending trusted identity/session attestation. Transported or
cached data never claims a profile was applied to a player.
