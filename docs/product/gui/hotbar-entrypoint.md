# Hotbar entrypoint

## Purpose

This contract defines the hard-locked local documentation token in slot `8`.

## Status

implemented

## Slot contract

- Slot `8` is the player hotbar index, not a raw inventory-view slot.
- Join, respawn, and player pickup repair the local `NETHER_STAR` token.
- The token carries only plugin-local persistent metadata and local display copy.
- Duplicate tokens outside slot `8` are removed during repair.
- The token is always present while the Paper plugin is enabled; no daemon-backed
  player setting enables, disables, or styles it.

## Open and movement rules

Use or drop intent for the token cancels the underlying event and opens bundled
documentation. Token clicks and cursor moves are cancelled and repaired; they do
not open a daemon route. Dragging into slot `8` is cancelled. The listener reads
only the current Bukkit inventory and never reads a credential, player profile,
database, filesystem, network, or process.

## Verification

Local adapter tests prove slot repair and token containment. They do not prove a
setting mutation, identity lookup, or daemon action.
