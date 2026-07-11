# GUI

## Purpose

This area owns the implemented menu engine product contract for inventory menus,
slot grammar, documentation browsing, hotbar entry, and player-facing failure
semantics.

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
- [Route catalog](routes/README.md)
- [Slot map](slot-map.md)
- [Style tokens](style-tokens.md)

## Contract

Menus are document-driven products. JSON documents define structure, the pure
kernel decides route and frame state, pure bindings decode data, and the Paper
runtime performs effects. Every visible item must either run a real action,
navigate through route history, or render an exact disabled reason. Decoration,
info panels, page indicators, and empty slots are inert and silent.

The bundled docs browser uses the local slot grammar: pagination `46`/`47`/`48`,
Back `49`, and Close `53` on 54-slot surfaces. The explicit Close slot is the
only menu action that may close an inventory. Root, dynamic, and confirmation
menus are withdrawn pending trusted identity/session attestation.

## Outcome, journey, and evidence boundary

A player opens bundled documentation, follows stable local navigation, and
invokes only metadata-bound local actions. Malformed or unavailable local data
preserves the session and renders safe localized feedback rather than performing
an inferred action. Pure kernel and adapter tests prove local route and failure
semantics; they do not claim a daemon-backed row is shipped.
