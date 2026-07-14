# Slot map

## Purpose

This document defines stable source slot and generated chrome rules.

## Status

implemented

## Layout

Route JSON owns every source slot. Fifty-four-slot routes reserve border and
chrome positions; 27-slot confirmations reserve the confirmation pair. Source
slots may not collide with chrome, pagination, or dynamic regions. Duplicate or
out-of-range slots fail compilation.

For 54-slot routes, Main Menu is `45`, Previous `46`, page info `47`, Next `48`,
Back `49`, Refresh `50`, and Close `53` when declared. Confirmation actions use
`11` and `15`; Close is `26`. Docs use these same controls and no alternate slot
map.

## Accessibility

Item name and lore contain non-color labels. Role and material reinforce but do
not replace localized text.
