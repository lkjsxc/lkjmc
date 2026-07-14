# GUI

## Purpose

This area owns the document-driven Paper inventory menu and slot-8 entrypoint.

## Status

planned

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

One source-owned engine renders all 62 indexed routes, including curated
player documentation. `/menu` and the hotbar token open `root`; `/docs` selects
documentation routes. Typed revisioned snapshots expose current, stale, or
unavailable data without fabricated rows.

Navigation and explicit Close are distinct. Session, route, request, render, and
slot metadata reject old or repeated input. Dynamic and mutation actions require
a current capability and trusted attestation; without both they deny with
localized fallback. This task adds no daemon mutation port.

## Evidence boundary

Goldens and a disposable protocol-like Paper/Folia inventory harness are
repeatable adapter evidence, not a live server or Minecraft client. External
Minecraft remains a later guarded lane.
