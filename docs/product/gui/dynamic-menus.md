# Dynamic menus

## Purpose

This document owns planned daemon-backed and local-source dynamic inventory
surfaces on the menu engine.

## Status

planned

## Phase model

Every dynamic route renders exactly one kernel phase:

- `Loading`: request in flight; chrome plus inert loading indicator.
- `Loaded`: binding returned a list, detail, or custom view.
- `Empty`: daemon or local source returned a valid empty result.
- `Denied`: the player lacks a required grant for the route or row set.
- `Stale`: last good view reused after a transient load failure.
- `Diagnostic`: typed failure with player-safe title and hint.

Loading, loaded, empty, denied, stale, and diagnostic replacement preserve route
history, session id, and selected route params except for the epoch increment
that protects old item metadata.

## Stale policy

The runtime keeps a bounded last-good view per player and route. On load failure
it may render stale data with a visible warning when the cache entry is fresh
enough. Stale frames keep navigation actions clickable, but daemon mutations are
rendered disabled with exact stale-copy so a player cannot mutate state based on
data that may be wrong.

## Domain surfaces

Travel uses daemon-backed homes, warps, teleport requests, and random-teleport
quotes. Choosing a home opens a detail route. Home overwrite and delete require
confirmation.

Economy uses points, shop, adventures, kits, votes, and daily reward data. Shop
rows show balance, price, post-purchase balance, category, delivery, and exact
disabled reason. Deterministic refund-safe item purchase may be direct.
Adventure purchase remains confirmed.

Profile and achievements show summaries, category paths, detail routes, and
claim buttons only when claimable. Server routes show desired state, observed
state, readiness, health, connect address, proxy registration, player count,
joinable flag, and exact disabled reason.

## Diagnostics

Bindings and effect runners emit only the diagnostic code vocabulary documented
in [failure semantics](failure-semantics.md). Valid empty data never renders as
an error, and a loader failure never renders as an empty list.

## Verification

Kernel tests cover every phase transition. Binding tests cover loaded, empty,
denied, stale, diagnostic, and disabled-row cases for each route family.
