# Design system

## Purpose

This contract defines the visual language for inventory menus.

## Size

Most player menus use 54 slots. Compact confirmation menus may use 27 slots
when the owner doc defines the flow.

## Stable controls

- Info panel: slot `4`.
- Previous page: slot `46`.
- Next page: slot `47`.
- Page info: slot `48`.
- Back: slot `49`.
- Refresh or close: slot `50`.
- Root close button: slot `50`.
- Functional slots override decoration.

## Border

Default 54-slot border slots are top row `0..8`, bottom row `45..53` except
functional controls, left column `9,18,27,36`, and right column `17,26,35,44`.

## Category panes

- Root: `LIGHT_BLUE_STAINED_GLASS_PANE`.
- Network and servers: `CYAN_STAINED_GLASS_PANE`.
- Travel, homes, and warps: `GREEN_STAINED_GLASS_PANE`.
- Economy, shop, kits, daily rewards, and votes: `YELLOW_STAINED_GLASS_PANE`.
- Claims and protection: `LIME_STAINED_GLASS_PANE`.
- Social, party, mail, and reports: `PURPLE_STAINED_GLASS_PANE`.
- Settings and language: `LIGHT_GRAY_STAINED_GLASS_PANE`.
- Dangerous confirmation: `RED_STAINED_GLASS_PANE`.

## Lore

Names and lore come from locale catalogs. Lore should state what will happen,
required permission when relevant, current state, cost, cooldown, daemon
availability, or disabled reason.
