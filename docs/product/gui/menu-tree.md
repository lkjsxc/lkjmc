# Menu tree

## Purpose

This contract defines the reachable player inventory menu hierarchy.

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

Documentation and Admin must never share a root slot. Admin may be visible to all
players as an entry point, but non-admin players must see disabled rows rather
than enabled dangerous actions.

## Required surfaces

Root links to network, travel, claims, economy, social, profile, settings,
documentation, admin, and adventures. Dynamic child surfaces cover servers,
homes, warps, teleport requests, claims, shop, adventure catalog, kits, daily
reward, votes, mail, reports, profile summaries, achievements, language,
personal settings, and permitted admin operations.

## Admin surface

Admin child menus group real daemon-backed operations:

- health: `/lkjmc status` and `/lkjmc doctor`;
- servers: list, create, start, stop, restart, and delete commands;
- config: check, reload, and restart warning commands;
- security: daemon-token status and rotation plus role catalog;
- economy: default seeding and shop catalog maintenance;
- moderation: reports, warnings, notes, bans, mutes, and claims inspection;
- audit: recent privileged events;
- web: authenticated web-control status guidance.

Rows without a real command, permission, or context render disabled localized
copy and must not silently close the inventory.

## Navigation

General product menus use route-stack Back history. Non-root menus render a
visible `menu.back` item in slot `49` that pops to the previous route. The docs
browser is the exception: it uses route-derived Parent Directory plus Main Menu
instead of menu Back history.

Dense lists use stable ordering, deterministic pagination, explicit refresh, and
true empty rows.

## Temporary adventures

Temporary adventures render cost, party size, time limit, risk, refund, and
return copy. Confirmation routes carry immutable purchase context and delegate to
the daemon adventure purchase command. Stale confirmations fail safely and do not
charge points.
