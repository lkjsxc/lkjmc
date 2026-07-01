# Navigation

## Purpose

This document owns route-stack navigation for inventory menus.

## Terms

- A route is a menu id plus route parameters needed to render that view.
- A route stack is the historical path for the player's open menu session.
- Root is the bottom route and is the only route after `OpenRoot`.
- Back means history Back. It pops the current route and renders the previous
  route; it is not Main Menu.
- `OpenRoute` means forward navigation or an explicit shortcut. It is not Back.
- Refresh means re-render the current route and, when dynamic, reload live data.

## Invariants

- The route stack is never empty.
- The current route always equals the stack's last route.
- `OpenRoute` pushes only when the target differs from the current route.
- `Back` never pushes.
- Refresh, loading replacement, and unavailable replacement never push or pop.
- Root has Close in slot `50` and no visible Back item.
- Main Menu shortcuts open root explicitly and use `NETHER_STAR`.
- Confirmation cancel uses true Back.

## Slot contract

A visible `menu.back` item in slot `49` must use `MenuAction.Back`. Parent-route
shortcuts must use another visible name and an explicit owner doc. Slot `49`
therefore consumes history in travel, teleports, claims, economy, social,
profile, settings, loading, unavailable, picker, and confirmation surfaces.

## Stack repair

If Back is clicked without a previous route, the adapter repairs to root with a
root-only stack. A pure parent graph may provide a direct-open fallback only for
documented repair cases; it must not override normal route history.

## Dynamic replacement

Dynamic menus may first render loading, then replace that inventory with loaded
or unavailable content. The replacement keeps the same route stack and changes
only render freshness metadata. Stale asynchronous responses from an older
session or render epoch are ignored.

## Verification

Pure tests cover stack operations and representative paths. Menu spec tests
assert that every non-root slot `49` item named `menu.back` uses
`MenuAction.Back`, including loading, unavailable, picker, travel, teleport,
claim, economy, social, profile, and settings menus.
