# GUI

## Purpose

This area owns the bounded local Paper documentation menu and hotbar token.

## Status

partial

Missing: trusted Java adapter identity and session attestation for daemon-backed
player data, actions, and effects.

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
- [Route catalog](routes/README.md)
- [Slot map](slot-map.md)
- [Style tokens](style-tokens.md)

## Contract

`/menu`, `/docs`, and the slot-8 token expose bundled documentation only. A
visible local item opens documentation, changes a local page, starts a local
search, or closes the inventory. No root menu, dynamic row, confirmation,
profile, admin, shop, exchange, claim, adventure, transfer, or player-data
operation is registered.

## Evidence boundary

Local menu and hotbar tests prove the bounded presentation surface. They do not
prove a daemon-backed menu, delivery, consent, or player mutation.
