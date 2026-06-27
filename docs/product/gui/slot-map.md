# Slot map

## Purpose

This document defines stable slot assignments for inventory menus.

## Global slots

- `4`: contextual info panel.
- `46`: previous page.
- `47`: next page.
- `48`: page info.
- `49`: route-stack Back on non-root menus; visible `menu.back` uses
  `MenuAction.Back` and never a parent `OpenRoute`.
- `50`: close on root or refresh on refreshable dynamic menus.

## Root slots

- `19`: Network and servers.
- `20`: Travel.
- `21`: Claims.
- `22`: Economy.
- `23`: Social.
- `24`: Profile and progression.
- `25`: Settings.
- `31`: Staff tools when permitted.
- `40`: Temporary adventures when documented.

## Border slots

Default 54-slot border slots are top row `0..8`, bottom row `45..53` except
functional controls, left column `9,18,27,36`, and right column `17,26,35,44`.
Functional slots win over border slots.

## Pagination slots

Growth-heavy lists render entries in the interior area only. Page controls are
stable even when a list has a single page; unavailable page buttons render as
disabled or inert with a clear reason.
