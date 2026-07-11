# Random teleport and portal policy

## Purpose

This contract owns daemon random-teleport profiles and the Java withdrawal
boundary.

## Status

implemented

## Profiles

The daemon owns `overworld`, `nether`, and `end` profiles. Quotes supply cost,
balance, cooldown, radius, attempts, affordability, confirmation requirement,
target environment, and world candidates.

## Current boundary

Paper `/rtp`, portal listeners, Travel menus, safe-location search, reservation,
refund, and player feedback are withdrawn pending trusted identity/session
attestation. A daemon quote or reservation does not claim that a player moved or
received a refund in Java.

## Safety policy

The daemon contract requires safe floors, passable feet and head, border
containment, and no hazardous exposure. Nether and End policies add their
respective lava, ceiling, void, island, and spawn-platform restrictions.

## Verification

Core and daemon tests cover profile quoting, cooldowns, safe-search failure, and
refund records. Java containment inspection proves portal and teleport adapters
are absent.
