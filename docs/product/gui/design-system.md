# Design system

## Purpose

This contract defines the shared visual language for inventory menus.

## Status

partial

Missing: every docs browser and dynamic menu path must use one shared chrome
helper with route-stack tests for Back, Parent Directory, Main Menu, Refresh,
and Close.

## Surface size

Player menus use 54 slots by default. Simple confirmations may use 27 slots when
the owner document names the compact surface. Rich confirmations and
browser-like routes use 54 slots.

## Shared chrome

All 54-slot surfaces share top, bottom, left, and right border panes unless an
owner document defines an exception. Border panes are inert, blank, silent, and
never carry action metadata. Functional controls override decoration.

The shared helper owns border slots, theme material, stable controls,
pagination, Back, Parent Directory, Main Menu, Refresh, and Close placement.
Normal menus, dynamic menus, unavailable menus, confirmations, docs browser,
settings, shop, achievements, and admin menus use that helper.

## Stable controls

- Slot `4`: contextual info panel unless intentionally empty.
- Slots `46`, `47`, and `48`: previous page, next page, and page info when a
  route paginates.
- Slot `49`: Back, or Parent Directory on documented browser routes.
- Slot `50`: Close on root menus and Refresh on dynamic routes.
- Main Menu uses `NETHER_STAR` and opens root explicitly.
- Back uses `ARROW` and consumes route history.

## Themes

Root uses light blue glass. Network uses cyan. Travel uses green. Claims use
lime. Economy uses yellow. Social uses purple. Profile and achievements use
orange. Settings use light gray. Staff uses red. Adventures use magenta.
Dangerous confirmations use red.

## Lore grammar

Interactive lore orders information as purpose, state, cost or reward, balance
or post-action balance, cooldown, permission or unlock, daemon or server
availability, and exact action phrase. Disabled lore states the exact reason,
next possible step, and whether the state is loading, unavailable, locked,
denied, stale, or temporary.

## Verification

Render tests cover root, settings, docs root, docs file, shop, achievements,
unavailable, and confirmation surfaces. Interaction tests prove decoration is
inert and silent.
