# Slot map

## Purpose

This document defines stable slot assignments for planned engine menus.

## Status

planned

## Global slots for 54-slot menus

- `4`: contextual info panel, inert.
- `45`: Main Menu using `NETHER_STAR`; opens root on non-root menus. Root uses
  decoration at this slot.
- `46`: previous page when pagination is declared.
- `47`: page indicator when pagination is declared; inert.
- `48`: next page when pagination is declared.
- `49`: route-stack Back, labeled Parent Directory on docs directory routes.
- `50`: Refresh exactly when the route has daemon data.
- `53`: Close on every engine menu.

The Close slot is the only menu action that closes the inventory. Single-page
lists still render the indicator row so controls never move; arrows at bounds
render disabled and inert.

## Root slots

- `4`: info.
- `19`: Network and servers.
- `20`: Travel.
- `21`: Claims.
- `22`: Economy.
- `23`: Social.
- `24`: Profile and progression.
- `25`: Settings.
- `30`: Documentation browser.
- `31`: Admin tools surface.
- `40`: Temporary adventures catalog.
- `53`: Close.

## Confirm slots

Compact confirmations use 27 slots: info `4`, confirm `11`, cancel `15`, and
close `26`. Confirm uses `LIME_WOOL` with success role. Cancel uses `RED_WOOL`
and the Back action.

## Border slots

Default 54-slot border slots are top row `0-8`, bottom row `45-53`, left column
`9`, `18`, `27`, `36`, and right column `17`, `26`, `35`, `44`. Functional
controls always win over border decoration.

## Regions

- `interior-28`: `10-16`, `19-25`, `28-34`, `37-43`.
- `interior-21`: `19-25`, `28-34`, `37-43`.
- `filter-row`: `10-16`.
- `detail-band`: `20-24`.
- `confirm-pair`: `11`, `15`.

## Docs browser exception

`docs-file` keeps reading controls next to content: previous page `21`, content
`22`, next page `23`, outbound links `52`, and Close `53`. This is the only
exception to the global pagination row.
