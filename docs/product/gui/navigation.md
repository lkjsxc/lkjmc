# Navigation

## Purpose

This document owns route-stack navigation, sessions, and page state for
implemented engine menus.

## Status

implemented

## Terms

- A route is a menu id plus route parameters needed to render that view.
- A route stack is the historical path for the player's open menu session.
- Root is the bottom route and is the only route after opening root.
- Back means history Back. It pops the current route and renders the previous
  route; it is not Main Menu.
- Refresh re-renders the current route and reloads data when the route is bound.

## Invariants

- The route stack is never empty and always has root at index `0`.
- The current route always equals the stack's last route.
- Opening root resets the stack to root only.
- Opening a different route id pushes.
- Opening the same route id with different params replaces the top route.
- Back pops and never pushes; it never pops below root.
- Refresh and phase replacement preserve stack and route.
- Cancel on a confirmation is true Back.
- No engine action closes the inventory except Close.
- Chosen object ids travel in route params and metadata.

## Session model

Session id, epoch, route, stack, phase, and page live in the kernel model. Every
route transition or render-affecting phase change bumps the epoch. Metadata
validation compares item session, epoch, route, params, and slot against the
current model so stale clicks cannot mutate state.

Manual player closes dispatch `InventoryClosed` and discard the session. Stale
asynchronous responses from an older session or epoch are ignored.

## Page and text input

Page position resets on route open. Refresh preserves page when the reloaded
list still has that page and clamps otherwise. Page turns on the same route use
the same-id replacement rule.

Text input records the prompt and command prefix against the current session,
waits for the next chat message, expires after 60 seconds, clears on quit, and
dispatches the text plus prefix back to the engine before refreshing the same
route. The prefix is a command literal without a trailing space; the kernel
normalizes one separator before the submitted text.

## Entry points

- `/menu` opens root.
- Hotbar `NETHER_STAR` right-click opens root.
- `/docs [path]` opens `docs-directory` or `docs-file`.
- Deep opens from commands target their route with stack `[root, target]` so
  Back lands on root.

## Verification

Pure tests cover stack operations, same-id replacement, page clamping, text
input session preservation, and stale metadata rejection.
