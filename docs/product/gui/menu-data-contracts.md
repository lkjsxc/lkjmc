# Menu data contracts

## Purpose

This file maps dynamic inventory routes to their data owner, schema, permission
state, empty state, and real effects.


## Status

implemented

## Route map

| Route | Loader or data command | Required shape | Permission state | Empty or disabled state | Enabled effects |
|---|---|---|---|---|---|
| `server-list` | `instance.list` | `instances[]` with `id`, `kind`, `desiredState`, `observedState`, `healthy`, optional `presence.playerCount` | Start and stop buttons check `lkjmc.admin.instance.start` and `lkjmc.admin.instance.stop` | `menu.server-list.empty`; stop disabled when occupied | `/lkjmc server start <id>`, `/lkjmc server stop <id>`, or save-first transfer to a joinable row target |
| `admin-servers` | `instance.list` | live rows with id, state, health, and presence | Server rows check `lkjmc.admin.instance.list` | Empty list keeps Create Server visible | Open selected-server detail or create flow |
| `admin-server-detail` | Route params plus `instance.list` refresh | selected `id` and latest state | Lifecycle rows check `lkjmc.admin.instance.*` | Missing id renders stale-route diagnostic | Open start, stop, restart, logs, or delete confirm |
| `admin-server-delete-confirm` | Route params | selected `id`, state, player count, `force` | Requires `lkjmc.admin.instance.delete` | Running or occupied rows disabled unless force is permitted | `instance.delete` body with exact `id` and `force` |
| `homes` | `player.home.list` | `homes[]` with `home`, `serverId` plus current location | Command parity requires `lkjmc.user.home` | `menu.homes.empty`; invalid names disabled; create stays visible in slot `45` | `/home <home>` or open generated-name confirmation |
| `warps` | `player.warp.list` | `warps[]` with `warp`, `serverId` | Command parity requires `lkjmc.user.warp` | `menu.warps.empty`; invalid command names disabled | `/warp <warp>` |
| `teleports` | Static route plus RTP quote | cost, cooldown, radius, attempts | Commands require teleport or RTP permissions | Picker empty when no online target; RTP disabled with exact reason | `/tpaccept`, picker to `/tpa <player>`, or RTP confirm |
| `random-teleport-confirm` | `player.random-teleport.quote` | cost, cooldown, balance, radius, attempts | Requires RTP permission | Disabled for cooldown, unaffordable, or disabled policy | `/rtp confirm` after quote |
| `teleport-picker` | Local online-player picker | Player names | Excludes the current player | `menu.player-picker.empty` | `/tpa <player>` |
| `home-create-name` | `player.home.list` and route location | next generated name: `home`, `home-2`, `home-3` | Requires `lkjmc.user.home` | Duplicate names are skipped | Open exact home create confirmation |
| `home-create-confirm` | Route params plus Paper location | generated name, server, nested `location` object | Requires `lkjmc.user.home` | Missing location is inert with a stale-location reason | `player.home.set` body with `serverId` and nested `location` |
| `claims` | `claim.list` | `claims[]` with `name`, `chunkCount` | Commands require `lkjmc.user.claim` | `menu.claims.empty` | Future generated-name create and detail routes |
| `claim-detail` | Route params | `name`, `chunkCount` | Commands require `lkjmc.user.claim` | Missing params render inert fallback text | Open delete confirm or trust picker |
| `claim-confirm` | Route params | `name` | Commands require `lkjmc.user.claim` | Missing name stays exact route context | `/claim delete <name>` |
| `claim-trust-picker` | Local online-player picker | Player names and claim name route param | Commands require `lkjmc.user.claim` | `menu.player-picker.empty` | `/claim trust <name> <player>` |
| `shop` | `player.shop.list` plus `player.points.balance` | `balance`; `items[]` with id, category, material, amount, price, executor, affordability | Commands require `lkjmc.user.shop` | `menu.shop.empty`; undeliverable or unaffordable items disabled with exact reason | `/buy <item>` only when affordable and delivery is supported |
| `kits` | `player.kit.list` | `kits[]` with `id`, `titleKey`, `rewardPoints`, `cooldownHours` | Commands require `lkjmc.user.kit` | `menu.kits.empty` | `/kit claim <kit>` |
| `votes` | `player.vote.list` | `links[]` with `id`, `titleKey`, `url` | Commands require `lkjmc.user.vote` | `menu.votes.empty` | `/vote <id>` |
| `daily` | `player.daily.status` | `claimedToday`, `points` | Commands require `lkjmc.user.daily` | Claimed state disabled | `/daily` when unclaimed |
| `mail` | `player.mail.inbox` | `messages[]` with `id`, `senderName`, `body`, `read` | Commands require `lkjmc.user.mail` | `menu.mail.empty` | `/mail read <id>` |
| `reports` | `player.report.list` | `reports[]` with `id`, `serverId`, `reason`, `status` | Hidden without `lkjmc.admin.reports` | `menu.reports.denied` or `menu.reports.empty` | Open report detail |
| `report-detail` | Route params | `reportId`, `serverId`, `reason`, `status` | Requires `lkjmc.admin.reports` for parent list | Missing params stay inert | Open resolve or dismiss confirm |
| `report-confirm` | Route params | `reportId`, `action` | Requires `lkjmc.admin.reports` | Missing params stay exact route context | `/reports resolve <id>` or `/reports dismiss <id>` |
| `profile` | `player.points.balance` and `player.achievements.list` | `balance`; `achievements[]` | Commands require profile-related user permissions | Loading or typed diagnostic | Open achievements, points, HUD routes |
| `achievements` | `player.achievements.list` | `achievements[]` with progress, state, and reward summaries | Command requires `lkjmc.user.achievements` | `menu.achievements.empty`; unavailable executor disabled | Open detail or claim a claimable reward |
| `party` | `player.party.info` | `found`, optional `name`, `role` | Commands require `lkjmc.user.party` | `menu.party.none` | One-click daemon create, invite picker, leave confirm |
| `party-confirm` | Static confirmation | Current party context | Commands require `lkjmc.user.party` | Cancel is Back | `/party leave` |
| `party-invite-picker` | Local online-player picker | Player names | Commands require `lkjmc.user.party` | `menu.player-picker.empty` | `/party invite <player>` |
| `settings` | Direct menu daemon commands | `player.settings.toggle` payloads | User setting permissions are enforced by daemon/command path | Disabled only on command failure | Toggle HUD and hotbar token |
| `language` | Direct menu daemon commands | `player.settings.set` payloads and cached resolved language | User language permission is enforced by daemon/command path | Disabled only on command failure | Persist `en` or `ja`, update cache, refresh current menu |

## Diagnostics

All loader failures map to localized diagnostics for missing daemon config, token
missing, token unreadable, HTTP failure, auth failure, unknown command, command
failure, database not configured, database unavailable, schema mismatch, or
permission denial. Diagnostics never print tokens, secret URLs, raw stack traces,
or generated secrets.
