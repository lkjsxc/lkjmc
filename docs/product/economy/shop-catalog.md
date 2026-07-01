# Shop catalog

## Purpose

This document defines the balanced default point shop catalog.

## Current status

The shop lists daemon items and delivers supported `minecraft-item` and generic
`adventure` metadata. The default catalog is seeded by
`lkjmc shop seed-defaults`, and core tests validate that seeded buy prices exceed
configured sell values for identical materials and amounts.

## Categories

- `building`
- `wood-nature`
- `food-farming`
- `redstone-utility`
- `travel-exploration`
- `claims-homes`
- `cosmetics`
- `adventures`
- `seasonal`
- `community`

## Delivery executors

- `minecraft-item`: material and amount delivered by the Paper adapter.
- `adventure`: purchase and start a catalog adventure by `adventureId`.

Unsupported executors remain disabled or fail before point deduction. Random
teleport is a daemon-quoted point sink outside the shop catalog and shares the
same ledger/refund discipline. Cosmetic or permission grant executors require
durable storage, expiry, scope, audit, and UI before they can become defaults.

## Balance rules

Buy prices must exceed exchange sell value for the same material and amount.
Defaults do not include diamonds, netherite, elytra, shulker boxes, spawners,
command-only items, or progression-breaking items unless an owner explicitly
configures them.
