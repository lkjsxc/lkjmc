# Menu tree

## Purpose

This contract defines the complete Paper menu hierarchy.

## Status

planned

## Root families

`root` reaches network, travel, claims, economy, social, profile, settings,
staff administration, adventures, and documentation. All 62 indexed routes are
reachable by parent or explicit navigation edges. Parent chains are acyclic and
terminate at `root`.

The documentation routes are `docs-directory`, `docs-file`, `docs-links`, and
`docs-search`. They are rendered by the same engine as every other route. `/docs`
selects one of these routes; `/menu` and the slot-8 token open `root`.

## Gates

A route remains selectable when a declared snapshot is unavailable so the
engine can state that failure truthfully. Stale and unavailable dependencies do
not expose mutation. Permission-dependent routes deny with localized copy when
a current exact grant is absent.

## Verification

The generated route catalog lists every route by theme. Graph validation and the
`all-routes-selected-engine` probe reject missing, unreachable, or alternate
runtime routes.
