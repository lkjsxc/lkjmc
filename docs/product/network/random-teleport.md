# Random teleport and portal policy

## Purpose

This contract owns random teleport dimension profiles and the replacement for
Nether and End portal travel on managed survival servers.

## Status

partial

Missing: daemon/store profile fields, paid Nether and End commands, dimension
aware safe search, menu routes, and refund tests for paid dimension teleports.

## Profiles

Random teleport has daemon-owned profiles:

- `overworld`: cost `0`, normal-world destination, no confirmation.
- `nether`: paid Nether destination, confirmation required.
- `end`: paid End destination, confirmation required.

Daemon quote responses are the only source for cost, balance, cooldown, radius,
attempts, affordability, confirmation requirement, target environment, and world
candidates. Menus and commands do not hardcode cost text.

## Command behavior

`/rtp` and `/rtp overworld` request the free overworld profile. `/rtp nether`
and `/rtp end` request paid quotes. `/rtp nether confirm` and `/rtp end confirm`
confirm a fresh paid quote. The Travel menu exposes the same profiles.

Safe-location search happens before point reservation. No safe location means no
charge. If a paid reservation succeeds and the final teleport fails, Paper calls
the daemon refund command once with the same correlation id.

## Portal policy

Nether and End portal events are cancelled and never spend points. The player
gets chat and action-bar guidance naming the replacement command and Travel menu
path. Portal entry must not trigger a reservation.

## Safety policy

Overworld search requires a solid floor, passable feet and head, world-border
containment, and no lava, fire, magma, cactus, powder snow, or void exposure.
Nether search also avoids lava pockets, fire, unsafe ceiling pockets, and
one-block ledges. End search avoids void exposure, unsafe islands, and spawn
platform collisions.

## Verification

Core tests cover all profiles. Daemon tests cover free quote, paid quote,
profile cooldowns, no-charge safe-search failure, and refund-on-final-failure.
Paper tests cover portal cancellation and profile world selection.
