# Hotbar entrypoint

## Purpose

This contract defines the slot `8` menu token used to open `/menu`.

## Slot contract

- Slot `8` means the player inventory hotbar index, not a raw view slot.
- Slot `8` is reserved only when the player's setting enables the token.
- Join and respawn restore the token.
- Inventory close and selected inventory mutation paths resync the token.
- The token carries persistent metadata and localized name and lore.
- Tokens outside slot `8` are stale duplicates and are removed.

## Open inputs

These inputs cancel the underlying event and open the root menu:

- right-click or left-click with the slot `8` token;
- air, block, or entity interaction with the slot `8` token;
- drop intent with the slot `8` token;
- direct inventory click of the slot `8` token while any inventory is open.

Success is silent except for opening the menu. Failure sends a concise localized
reason and does not unlock slot `8`.

## Movement lock

These inputs are cancelled and resynced:

- dragging into the player hotbar slot `8`;
- number-key swaps involving hotbar slot `8`;
- offhand swaps involving the token;
- transfer vectors involving the token;
- moving stale duplicate tokens.

Number-key swaps do not open the menu. Non-token items do not open the menu;
resync replaces them with the token when the setting is enabled.
