# Design system

## Purpose

This contract defines the polished visual language for player inventory menus.

## Surface size

Player menus use 54 slots by default. Dedicated confirmation menus may use 27
slots when the destructive flow owner documents the compact surface.

## Stable controls

- Slot `4`: contextual info panel unless an owner doc leaves it empty.
- Slots `46`, `47`, and `48`: previous page, next page, and page info.
- Slot `49`: route-based back on non-root menus.
- Slot `50`: close on root; refresh only where fresh data is useful.
- Functional slots override borders and decoration.

## Category themes

- Root: `LIGHT_BLUE_STAINED_GLASS_PANE`.
- Network and servers: `CYAN_STAINED_GLASS_PANE`.
- Travel, homes, warps, and teleports: `GREEN_STAINED_GLASS_PANE`.
- Claims and protection: `LIME_STAINED_GLASS_PANE`.
- Economy, shop, kits, daily rewards, and votes: `YELLOW_STAINED_GLASS_PANE`.
- Social, party, mail, and reports: `PURPLE_STAINED_GLASS_PANE`.
- Profile and progression: `ORANGE_STAINED_GLASS_PANE`.
- Settings and language: `LIGHT_GRAY_STAINED_GLASS_PANE`.
- Staff and moderation: `RED_STAINED_GLASS_PANE`.
- Temporary adventures: `MAGENTA_STAINED_GLASS_PANE`.
- Dangerous confirmation: `RED_STAINED_GLASS_PANE`.

## Border grammar

Default 54-slot border slots are top row `0..8`, bottom row `45..53` except
functional controls, left column `9,18,27,36`, and right column `17,26,35,44`.
Every border pane is inert, has a blank display name, and has no lore.

## Lore grammar

Interactive lore uses this order when relevant: purpose, current state, cost or
reward, cooldown or grace time, required permission or unlock, daemon or server
availability, then an exact action phrase such as `Click to open homes`.
Disabled lore states the exact reason, what the player can do next, and whether
the state is loading, unavailable, locked, denied, or temporary.
