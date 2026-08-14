# Backend menu

## Purpose

This area owns the small local Paper/Folia inventory menu and slot-8 entrypoint.

## Status

implemented, not player-accepted

## Supported routes

The compiled bundle contains exactly five routes:

- `root`, with inert `/lkjmc` command guidance and a link to docs;
- `docs-directory`;
- `docs-file`;
- `docs-links`;
- `docs-search`.

The only effects are local navigation, Back, and explicit Close. There is no
remote refresh, mutation, confirmation, snapshot subscription, or daemon command
path.

## Documents

- [Documentation browser](docs-browser.md)
- [Hotbar entrypoint](hotbar-entrypoint.md)
- [Interaction contract](interaction-contract.md)
- [Generated route catalog](routes/README.md)

## Evidence boundary

Goldens and deterministic Paper adapter probes render all five routes and inspect
the shaded jar. They are not a live server or Minecraft client. Player menu
acceptance remains outstanding.
