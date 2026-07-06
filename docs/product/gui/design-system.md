# Design system

## Purpose

This contract defines the planned visual language for engine-rendered inventory
menus.

## Status

planned

## Surface size

Player menus use 54 slots by default. Simple confirmations use 27 slots when
the document kind is `confirm`. Rich confirmations and browser-like routes use
54 slots.

## Shared chrome

All 54-slot surfaces share top, bottom, left, and right border panes. Border
panes are inert, blank, silent, and carry inert metadata. Functional controls
override decoration. The document declares which controls appear; the renderer
owns materials and placement.

Global controls are stable: info `4`, Main Menu `45`, previous page `46`, page
indicator `47`, next page `48`, Back or Parent Directory `49`, Refresh `50`,
and Close `53`. Close is present on every engine menu and is the only closing
action. Root uses decoration at `45`, no Back, no Refresh, and Close `53`.

## Regions

Documents reference named regions instead of raw slot ranges:

- `interior-28`: `10-16`, `19-25`, `28-34`, `37-43`.
- `interior-21`: `19-25`, `28-34`, `37-43`.
- `filter-row`: `10-16`.
- `detail-band`: `20-24`.
- `confirm-pair`: `11`, `15`.

## Themes

Theme names map to border panes in one renderer table: `root` light blue,
`network` cyan, `travel` green, `claims` lime, `economy` yellow, `social`
purple, `profile` orange, `settings` light gray, `staff` red, `adventure`
magenta, `danger` red, and `docs` brown.

## Lore grammar

Interactive lore orders information as purpose, state, cost or reward, balance
or post-action balance, cooldown, permission or unlock, daemon or server
availability, and exact action phrase. Disabled lore states the exact reason,
next possible step, and whether the state is loading, unavailable, locked,
denied, stale, temporary, or tied to a diagnostic code.

## Verification

Render tests cover root, settings, docs directory, docs file, shop,
achievements, diagnostic, stale, and confirmation surfaces. Interaction tests
prove decoration is inert and silent.
