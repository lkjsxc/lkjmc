# Random teleport and portal policy

## Purpose

This contract owns paid random teleport and the replacement for Nether and End
portal travel on managed survival servers.


## Status

implemented

## Player behavior

`/rtp` and the Travel menu expose Random Teleport. The visible quote shows cost,
cooldown, world, radius, attempts, and whether the player can afford it before a
teleport is attempted.

Confirming first searches for a safe destination. If no safe location is found,
no points are charged. If points are charged and the final teleport fails, the
daemon refunds once using a refund ledger correlation derived from the reservation
correlation id and returns an exact localized reason.

## Defaults

The daemon-owned default policy is cost `250`, cooldown `10m`, minimum radius
`750`, maximum radius `5000`, attempts `64`, and current overworld only. Menus
and messages read these values from daemon quote responses instead of embedding
numbers in static text.

## Portal policy

Player Nether and End portal transfers are cancelled. Entering a portal never
charges points. The player receives localized guidance that portals are disabled
and that `/rtp` or Travel > Random Teleport is the supported replacement.
Non-player entity portal behavior remains disabled until a real owner contract
needs it.

## Safety policy

A destination is valid only when it is inside the world border, in an allowed
normal world, in a loaded target chunk, above a solid floor, has passable feet and
head blocks, and avoids lava, fire, magma, cactus, powder snow, and void
exposure. Claim-aware exclusion is future until a scheduler-safe query exists.

## Verification

Default verification covers policy, store, daemon catalog, command metadata,
menus, permissions, and locale parity. Live smoke should enter Nether and End
portals to prove cancellation, then run `/rtp confirm` with enough points to
prove safe teleport and refund-on-failure diagnostics.
