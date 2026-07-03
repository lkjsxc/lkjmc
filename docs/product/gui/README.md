# GUI

## Purpose

This area owns inventory menu contracts, slot maps, hotbar entrypoint behavior,
and player-facing failure semantics.


## Status

implemented

## Table of contents

- [Action bar](action-bar.md)
- [Confirmation policy](confirmation-policy.md)
- [Design system](design-system.md)
- [Documentation browser](docs-browser.md)
- [Dynamic menus](dynamic-menus.md)
- [Failure semantics](failure-semantics.md)
- [Hotbar entrypoint](hotbar-entrypoint.md)
- [HUD setting](hud.md)
- [Interaction contract](interaction-contract.md)
- [Inventory sync](inventory-sync.md)
- [Menu data contracts](menu-data-contracts.md)
- [Menu framework](menu-framework.md)
- [Menu tree](menu-tree.md)
- [Navigation](navigation.md)
- [Slot map](slot-map.md)
- [Style tokens](style-tokens.md)

## Contract

Menus are metadata-driven products, not display-name command wrappers. Every
visible item must either perform a real action without unintended closing,
navigate by route-stack history, or render an exact disabled reason. Decoration,
info panels, page indicators, and empty slots are inert and silent.
