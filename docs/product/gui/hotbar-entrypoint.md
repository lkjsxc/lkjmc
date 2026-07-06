# Hotbar entrypoint

## Purpose

This contract defines the hard-locked slot `8` menu token and how it opens the
implemented menu engine.

## Status

implemented

## Slot contract

- Slot `8` means the player hotbar index, not raw view slot `8`.
- The token is present only when the daemon-backed setting enables it.
- The token material is `NETHER_STAR` and carries stable plugin metadata plus
  localized MiniMessage-rendered copy.
- Tokens outside slot `8` are stale duplicates and are removed.
- Join, respawn, inventory close, pickup completion, and blocked movement paths
  schedule repair passes.

## Open inputs

Item use, entity interaction, drop intent, and direct inventory click on the
real token cancel the underlying event and call `UiEntrypoints.openRoot` on the
player scheduler. Opening failure sends a localized reason and keeps the token
locked.

## Movement lock

Dragging into slot `8`, number-key swaps involving slot `8`, offhand swaps with
the token, transfer vectors involving plugin tokens, and moving stale duplicate
tokens are cancelled and repaired. Number-key swaps do not open root unless the
source item is the token and the owner doc explicitly allows that behavior.

## Ordinary items

Non-token items do not open root. When the token setting is enabled, slot `8` is
repaired back to the token after ordinary inventory activity completes. When the
setting is disabled, stale tokens are removed and slot `8` is left available.
