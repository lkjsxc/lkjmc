# Menu tree

## Purpose

This contract defines the reachable player inventory menu hierarchy.

## Status

partial

Missing: home detail routes, achievement directory/detail routes, paid dimension
random-teleport routes, and full shared chrome use in docs browser.

## Root layout

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
- Slot `50`: Close.

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

Product menus use route-stack Back. Non-root menus render slot `49` Back unless
the route is a documented browser surface, where slot `49` is Parent Directory.
Main Menu shortcuts use `NETHER_STAR` and open root explicitly. Cancel on a
confirmation is true Back.

Dense lists use stable ordering, pagination, explicit refresh, and true empty
rows. Selected server, report, claim, home, shop, achievement, or player ids stay
in route params or metadata. Players do not retype selected ids or confirmation
tokens when a route already carries that context.

## High-risk surfaces

Server stop, restart, delete, create-and-start, paid dimension random teleport,
adventure purchase, home delete, and home location overwrite require
confirmation. Safe navigation, settings toggles, home teleport, free overworld
random teleport, deterministic shop purchase, and idempotent reward claims do
not.

## Verification

Route tests cover Back, Parent Directory, Main Menu, confirmation cancel, and
metadata payload preservation for selected ids.
