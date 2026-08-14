# Hotbar entrypoint

## Purpose

This contract defines the hard-locked local menu token in player hotbar slot `8`.

## Slot contract

- Slot `8` is the player hotbar index, not a raw inventory-view slot.
- Join, respawn, and pickup repair the local `NETHER_STAR` token.
- The token carries only plugin-local persistent metadata and display copy.
- Duplicate tokens outside slot `8` are removed during repair.

Use or drop intent cancels the underlying event and opens `root`. Token clicks,
cursor moves, and drags into slot `8` are cancelled and repaired. Repair reads
only current Bukkit inventory state. Opening the menu does not read a remote
snapshot or dispatch a command.

## Verification

Paper tests assert the slot constant and local repair rules. Menu probes exercise
the same root renderer; this remains protocol-like evidence rather than a live
player observation.
