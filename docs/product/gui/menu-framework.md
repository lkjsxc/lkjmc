# Menu framework

## Purpose

This document defines the target reusable inventory UI framework.

## Domain types

- `MenuId`
- `MenuSpec`
- `MenuTitle`
- `MenuSize`
- `SlotSpec`
- `ItemSpec`
- `MenuAction`
- `MenuState`
- `MenuContext`
- `MenuDecision`
- `MenuEffect`
- `Pagination`
- `NavigationPolicy`

## Reducers

`render(menu, context)` returns an inventory model. `click(menu, state, click)`
returns a decision with effects.

## Current status

Java common implements platform-neutral menu records, immutable slot lists,
click decisions, pagination, confirmation specs, root/settings/language menu
factories, and basic effects. Reducers cover command, open menu, refresh, close,
and confirmation actions without importing Bukkit, Paper, Folia, or Velocity
APIs. The Paper adapter opens the root menu and provides the hotbar entry item
behind listener guardrails.
