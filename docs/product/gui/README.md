# GUI

## Purpose

This area owns inventory menu contracts, slot maps, hotbar entrypoint behavior,
and player-facing failure semantics.

## Table of contents

- [Design system](design-system.md)
- [Dynamic menus](dynamic-menus.md)
- [Failure semantics](failure-semantics.md)
- [Hotbar entrypoint](hotbar-entrypoint.md)
- [Interaction contract](interaction-contract.md)
- [Inventory sync](inventory-sync.md)
- [Menu framework](menu-framework.md)
- [Menu tree](menu-tree.md)
- [Slot map](slot-map.md)
- [Style tokens](style-tokens.md)

## Contract

Menus are metadata-driven products, not display-name command wrappers. Every
visible item must either perform a real action, navigate, or render an exact
disabled reason. Decoration, info panels, page indicators, and empty slots are
inert and silent.
