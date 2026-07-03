# Menu data contracts

## Purpose

This file maps dynamic routes to data owners, schemas, empty states, and real
effects.

## Status

partial

Missing: route implementations for home detail, home update/delete
confirmation, achievement directories/details, random-teleport profiles, and
proxy-registration joinability fields.

## Core route rules

Every loader returns one of loaded, true empty, permission denied, typed
diagnostic, or stale loaded data with warning. Enabled effects require concrete
ids in metadata and a real adapter. Disabled rows keep exact reason metadata and
do not perform command parity.

## Route map

| Route | Data command | Required shape | Enabled effects |
|---|---|---|---|
| `server-list` | `instance.list` | instances with state, health, heartbeat, connect address, proxy registration, joinable, disabled reason | save-first transfer only when joinable |
| `admin-servers` | `instance.list` | live rows plus create availability | open detail or create plan |
| `admin-server-detail` | `instance.list` | selected id, lifecycle state, logs hint | start, stop, restart, delete confirms |
| `homes` | `player.home.list` | homes with name, server id, location summary | open `home-detail`, create generated home |
| `home-detail` | `player.home.get` | selected home, server id, location | teleport, open update confirm, open delete confirm |
| `home-update-confirm` | route plus current location | home name, old and new locations | `player.home.set` overwrite |
| `home-delete-confirm` | route selected home | home name and owner | `player.home.delete` |
| `warps` | `player.warp.list` | warps with name and server id | `/warp <name>` command parity |
| `teleports` | RTP quote and local players | profile quotes and online player names | open profile quote, `/tpa`, `/tpaccept` |
| `random-teleport-confirm` | `player.random-teleport.quote` | profile id, cost, balance, cooldown, radius, attempts, confirmation | paid profile confirm or free direct RTP |
| `shop` | `player.shop.list` plus balance | balance and items with price, delivery, affordability, disabled reason | `/buy <item>` only when supported and refund-safe |
| `achievements` | `player.achievements.list` | summary and category paths | open achievement directory or detail |
| `achievement-directory` | cached list | path, child directories, visible achievements | open child or detail |
| `achievement-detail` | cached list | id, progress, criteria, reward, state | claim when claimable |
| `settings` | daemon settings commands | current language, Action Bar, hotbar token | reversible toggles without confirmation |

Other routes keep their documented data owners and must follow the same empty,
stale, diagnostic, permission, and exact-effect rules.

## Verification

Menu tests cover every route that opens a child route, every confirmation route,
and every disabled row class. Command-doc drift checks keep route effect commands
aligned with the command catalog.
