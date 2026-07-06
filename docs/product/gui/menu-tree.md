# Menu tree

## Purpose

This contract defines the planned reachable player inventory menu hierarchy.

## Status

planned

## Root layout

The root document is a 54-slot static route with close-only chrome:

- Slot `4`: network and player info.
- Slot `19`: Network and servers.
- Slot `20`: Travel.
- Slot `21`: Claims.
- Slot `22`: Economy.
- Slot `23`: Social.
- Slot `24`: Profile and progression.
- Slot `25`: Settings.
- Slot `30`: Documentation browser.
- Slot `31`: Admin tools surface.
- Slot `40`: Temporary adventures catalog.
- Slot `53`: Close.

Documentation and Admin never share a root slot. Admin may be visible to all
players as an entry point, but non-admin players see disabled rows rather than
enabled dangerous actions.

## Required surfaces

Root links to network, travel, claims, economy, social, profile, settings,
documentation, admin, and adventures. Dynamic children cover servers, homes,
selected home detail, warps, random-teleport profiles, teleport requests,
claims, shop, kits, daily reward, votes, mail, reports, profile summaries,
achievement directories, achievement details, language, personal settings, and
permitted admin operations.

## Navigation

Product menus use route-stack Back at slot `49`. Docs directory routes use the
same action labeled Parent Directory. Main Menu uses `NETHER_STAR` at slot `45`
and opens root explicitly. Cancel on a confirmation is true Back.

Opening a different route id pushes. Opening the same route id with different
params replaces the top stack entry. This keeps filter switches and page turns
from inflating Back history while preserving selected ids in route params.

## High-risk surfaces

Server stop, restart, delete, create-and-start, paid dimension random teleport,
adventure purchase, home delete, and home location overwrite require
confirmation. Safe navigation, settings toggles, home teleport, free overworld
random teleport, deterministic shop purchase, and idempotent reward claims do
not.

## Verification

Route tests cover Back, Parent Directory, Main Menu, same-id replacement,
confirmation cancel, and metadata payload preservation for selected ids.
