# Menu tree

## Purpose

This contract defines the reachable player menu hierarchy.

## Root layout

- Slot `4`: network and player info.
- Slot `19`: Network and servers.
- Slot `20`: Travel.
- Slot `21`: Claims.
- Slot `22`: Economy.
- Slot `23`: Social.
- Slot `24`: Profile and progression.
- Slot `25`: Settings.
- Slot `31`: Staff tools when permitted; otherwise empty or disabled by owner.
- Slot `40`: Temporary adventures with End Expedition status and purchase
  controls when daemon availability allows them.
- Slot `50`: Close.

## Required surfaces

Root links to network, travel, claims, economy, social, profile, and settings.
Dynamic child surfaces cover servers, homes, warps, teleport requests, claims,
shop, End Expedition, kits, daily reward, votes, mail, reports, profile
summaries, achievements, language, and personal settings when daemon-backed data
and actions exist.

## Navigation

Back is route-stack history. Non-root menus render a visible `menu.back` item in
slot `49` that pops to the previous route. The item never opens a parent route
by `OpenRoute`. Dense lists use stable ordering, deterministic pagination, and
explicit refresh.

## Temporary adventures

Temporary adventures render cost, party size, time limit, risk, and refund copy.
Solo and party confirmation routes carry immutable purchase context and delegate
to the daemon adventure purchase command. Stale confirmations fail safely and do
not charge points.
