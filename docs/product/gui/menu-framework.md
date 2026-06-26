# Menu framework

## Purpose

This document defines the target reusable inventory UI framework.

## Domain types

- `MenuId`
- `MenuSpec`
- `MenuTitle`
- `MenuSize`
- `MenuSlot`
- `ItemSpec`
- `ItemLore`
- `ItemVisualRole`
- `MenuAction`
- `MenuActionPayload`
- `MenuState`
- `MenuContext`
- `MenuClick`
- `MenuDecision`
- `MenuEffect`
- `Pagination`
- `NavigationPolicy`
- `MenuTheme`
- `MenuRegistry`
- `MenuRoute`

## Reducers

`render(menu, context)` returns an inventory model. `click(menu, state, click)`
returns a decision with effects. Reducers are pure and must not import Bukkit,
Paper, Folia, Velocity, network, database, filesystem, or process APIs.

## Target behavior

Java common implements platform-neutral menu records, immutable slot lists,
click decisions, pagination, confirmation specs, visual roles, themes, menu
registry, and standard menu factories. Reducers classify action, navigation,
inert, empty, outside, and unknown metadata clicks without importing Bukkit,
Paper, Folia, or Velocity APIs.

The Paper adapter renders metadata-bearing items, tracks sessions, cancels plugin
menu input before reduction, and executes returned effects through scheduler-safe
adapters.
