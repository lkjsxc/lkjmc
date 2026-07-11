# Adventure catalog

## Purpose

This document defines the default temporary adventure catalog.


## Status

implemented

## Definition fields

Each definition has id, title key, lore key, icon material, category, price,
party-size bounds, max lifetime, retention, runtime kind, jar kind, world
profile, cleanup policy, permission, and enabled diagnostic.

## Seeded adventures

- `end-expedition`: temporary End challenge.

Generic generated-world labels are not catalog entries. They remain withdrawn
until each has distinct implemented objectives, rules, and completion effects.

## Invariants

Ids are stable kebab-case strings, costs are positive, party bounds are valid,
world profiles map to real temporary instance allocation, startup failure refunds
exactly once, and cleanup policy is enforced by the existing cleanup worker.
