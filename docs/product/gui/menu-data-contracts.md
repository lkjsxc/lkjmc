# Menu data contracts

## Purpose

This file maps implemented dynamic routes to bindings, daemon commands, empty
states, and real effects.

## Status

implemented

## Core route rules

Every binding returns loaded, true empty, permission denied, typed diagnostic,
or stale loaded data with warning. Enabled effects require concrete ids in
metadata and a real adapter. Disabled rows keep exact reason metadata and do not
perform command parity.

Local-source bindings resolve in the dispatch path and do not show artificial
loading. Daemon-source bindings return a kernel request plan; the runtime sends
it asynchronously and decodes the response before dispatching data messages.

## Route to binding map

| Route family | Binding | Commands read | Enabled effects |
|---|---|---|---|
| `server-list` | `servers` | `instance.list` | save-first transfer when joinable |
| `admin-servers` | `admin-servers` | `instance.list` | open detail or create plan |
| `admin-server-detail` | `admin-server-detail` | `instance.list`, `instance.logs` | start, stop, restart, delete confirms |
| `admin-create` | `admin-create` | `instance.create.plan` | create and start when confirmed |
| `homes` | `homes` | `player.home.list` | open detail, create generated home |
| `home-detail` | `home-detail` | `player.home.get` | teleport, update confirm, delete confirm |
| `home-update-confirm` | `home-confirm` | current route context | `player.home.set` overwrite |
| `home-delete-confirm` | `home-confirm` | current route context | `player.home.delete` |
| `warps` | `warps` | `player.warp.list` | run `/warp <name>` command parity |
| `teleports` | `teleports` | `player.random-teleport.quote` | open quote, `/tpa`, `/tpaccept` |
| `random-teleport-confirm` | `random-teleport` | `player.random-teleport.quote` | reserve or complete profile |
| `shop` | `shop` | `player.shop.list`, `player.points.balance` | `/buy <item>` command delivery when safe |
| `adventures` | `adventures` | `adventure.catalog.list` | open purchase confirm |
| `achievements` | `achievements` | `player.achievements.list` | open directory or detail |
| `achievement-detail` | `achievement-detail` | `player.achievements.list` | `player.achievement.claim` |
| `settings` | `settings` | `player.settings.get` | setting commands without confirmation |
| `docs-directory` | `docs-directory` | local docs bundle | open child, prompt search |
| `docs-file` | `docs-file` | local docs bundle | page turn, open links |
| `docs-links` | `docs-links` | local docs bundle | open internal link, send external link |
| `docs-search` | `docs-search` | local docs bundle | open matching file |

The generated route catalog under `docs/product/gui/routes/` enumerates every
route, kind, binding, data command, and confirmation reason once the menu
documents exist. Hand-authored docs define behavior; generated docs list
inventory.

## Required response shapes

Bindings may read only fields asserted by daemon shape tests. Server rows must
include state, health, heartbeat, connect address, proxy registration, joinable,
player count, and disabled reason. Shop rows must include balance, price,
delivery, affordability, and disabled reason. Settings rows must include current
language, Action Bar, and hotbar token state.

## Verification

Menu tests cover every route that opens a child route, every confirmation route,
and every disabled row class. Contract checks keep route effect commands aligned
with the command catalog.
