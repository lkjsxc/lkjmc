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
- `nether-fortress-raid`: blaze and fortress survival route.
- `ancient-city-delve`: stealth sculk route with Warden risk.
- `trial-vault-run`: combat route around trial chambers.
- `ocean-monument-dive`: underwater guardian route.
- `woodland-mansion-hunt`: mansion objective route.
- `sky-island-rush`: floating island timed route.
- `resource-rush`: short renewable resource world isolated from main worlds.

## Invariants

Ids are stable kebab-case strings, costs are positive, party bounds are valid,
world profiles map to real temporary instance allocation, startup failure refunds
exactly once, and cleanup policy is enforced by the existing cleanup worker.
