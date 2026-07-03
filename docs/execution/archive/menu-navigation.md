# Menu navigation

## Purpose

This task tracks the route-stack Back repair for inventory menus.


## Status

completed

## Contract work

- Add a dedicated GUI navigation owner doc.
- Align menu tree, framework, slot map, interaction, dynamic, and failure docs.
- Keep current-state unchanged until code and verification prove the behavior.

## Implementation evidence

- Common Java owns stack invariants in pure navigation code.
- Every visible slot `49` item named `menu.back` uses `MenuAction.Back`.
- Refresh, loading replacement, and unavailable replacement preserve stacks.
- Paper keeps scheduler, daemon, and inventory effects in the adapter.

## Verification

Common menu tests cover route-stack paths and Back slot invariants. Final
handoff must list the exact full gates run in the current work session.
