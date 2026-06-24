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

No menu code is implemented yet.
