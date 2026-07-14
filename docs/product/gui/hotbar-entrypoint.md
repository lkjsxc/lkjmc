# Hotbar entrypoint

## Purpose

This contract defines the hard-locked menu token in slot `8`.

## Status

implemented

## Slot contract

- Slot `8` is the player hotbar index, not a raw inventory-view slot.
- Join, respawn, and player pickup repair the local `NETHER_STAR` token.
- The token carries only plugin-local persistent metadata and display copy.
- Duplicate tokens outside slot `8` are removed during repair.

## Open and movement rules

Use or drop intent cancels the underlying event and opens the `root` route in
the selected menu engine. Token clicks and cursor moves are cancelled and
repaired. Dragging into slot `8` is cancelled. Repair reads only current Bukkit
inventory state; opening may consume immutable A-JVM snapshot views but never
blocks the scheduler thread.

A settings snapshot can render menu preferences, but no unattested settings
mutation changes token ownership. The token never dispatches a generic command.

## Verification

Paper tests assert the slot constant. Menu probes cover selected-engine opening;
the protocol-like harness is not a live player.
