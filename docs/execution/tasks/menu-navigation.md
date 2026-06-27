# Menu navigation

## Purpose

This task tracks the route-stack Back repair for inventory menus.

## Contract work

- Add a dedicated GUI navigation owner doc.
- Align menu tree, framework, slot map, interaction, dynamic, and failure docs.
- Keep current-state unchanged until code and verification prove the behavior.

## Implementation work

- Move stack invariants into common pure navigation code.
- Make every visible slot `49` item named `menu.back` use `MenuAction.Back`.
- Preserve stacks across refresh, loading replacement, and unavailable
  replacement.
- Keep Paper scheduler, daemon, and inventory effects in the adapter.

## Verification

Run common menu tests while iterating, then run repository line, docs, locale,
Java, default verify, and Compose verify gates when available.
