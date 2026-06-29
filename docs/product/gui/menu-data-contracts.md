# Menu data contracts

## Purpose

This file maps dynamic inventory routes to their data owner, schema, permission
state, empty state, and real effects.

## Route map

| Route | Loader or data command | Required shape | Permission state | Empty or disabled state | Enabled effects |
|---|---|---|---|---|---|
| `server-list` | `instance.list` | `instances[]` with `id`, `kind`, `desiredState`, `observedState`, `healthy`, optional `presence.playerCount` | Start and stop buttons check `lkjmc.admin.instance.start` and `lkjmc.admin.instance.stop` | `menu.server-list.empty`; stop disabled when occupied | `/lkjmc server start <id>` or `/lkjmc server stop <id>` |
| `homes` | `player.home.list` | `homes[]` with `home`, `serverId` | Command parity requires `lkjmc.user.home` | `menu.homes.empty`; invalid command names disabled | `/home <home>` |
| `warps` | `player.warp.list` | `warps[]` with `warp`, `serverId` | Command parity requires `lkjmc.user.warp` | `menu.warps.empty`; invalid command names disabled | `/warp <warp>` |
| `teleports` | Static route | None | Commands require `lkjmc.user.teleport.request` | Picker empty when no online target | `/tpaccept`, picker to `/tpa <player>` |
| `teleport-picker` | Local online-player picker | Player names | Excludes the current player | `menu.player-picker.empty` | `/tpa <player>` |
| `claims` | `claim.list` | `claims[]` with `name`, `chunkCount` | Commands require `lkjmc.user.claim` | `menu.claims.empty` | Text input for `/claim create <name>` and detail routes |
| `claim-detail` | Route params | `name`, `chunkCount` | Commands require `lkjmc.user.claim` | Missing params render inert fallback text | Open delete confirm or trust picker |
| `claim-confirm` | Route params | `name` | Commands require `lkjmc.user.claim` | Missing name stays exact route context | `/claim delete <name>` |
| `claim-trust-picker` | Local online-player picker | Player names and claim name route param | Commands require `lkjmc.user.claim` | `menu.player-picker.empty` | `/claim trust <name> <player>` |
| `shop` | `player.shop.list` | `items[]` with `id`, `titleKey`, `pricePoints`, `deliveryAvailable` | Commands require `lkjmc.user.shop` | `menu.shop.empty`; undeliverable item disabled | `/buy <item>` only when delivery is supported |
| `kits` | `player.kit.list` | `kits[]` with `id`, `titleKey`, `rewardPoints`, `cooldownHours` | Commands require `lkjmc.user.kit` | `menu.kits.empty` | `/kit claim <kit>` |
| `votes` | `player.vote.list` | `links[]` with `id`, `titleKey`, `url` | Commands require `lkjmc.user.vote` | `menu.votes.empty` | `/vote <id>` |
| `daily` | `player.daily.status` | `claimedToday`, `points` | Commands require `lkjmc.user.daily` | Claimed state disabled | `/daily` when unclaimed |
| `mail` | `player.mail.inbox` | `messages[]` with `id`, `senderName`, `body`, `read` | Commands require `lkjmc.user.mail` | `menu.mail.empty` | `/mail read <id>` |
| `reports` | `player.report.list` | `reports[]` with `id`, `serverId`, `reason`, `status` | Hidden without `lkjmc.admin.reports` | `menu.reports.denied` or `menu.reports.empty` | Open report detail |
| `report-detail` | Route params | `reportId`, `serverId`, `reason`, `status` | Requires `lkjmc.admin.reports` for parent list | Missing params stay inert | Open resolve or dismiss confirm |
| `report-confirm` | Route params | `reportId`, `action` | Requires `lkjmc.admin.reports` | Missing params stay exact route context | `/reports resolve <id>` or `/reports dismiss <id>` |
| `profile` | `player.points.balance` and `player.achievements.list` | `balance`; `achievements[]` | Commands require profile-related user permissions | Loading or typed diagnostic | Open achievements, points, HUD routes |
| `achievements` | `player.achievements.list` | `achievements[]` with `id`, `titleKey` | Command requires `lkjmc.user.achievements` | `menu.achievements.empty` | Informational rows only |
| `party` | `player.party.info` | `found`, optional `name`, `role` | Commands require `lkjmc.user.party` | `menu.party.none` | Text input create, invite picker, leave confirm |
| `party-confirm` | Static confirmation | Current party context | Commands require `lkjmc.user.party` | Cancel is Back | `/party leave` |
| `party-invite-picker` | Local online-player picker | Player names | Commands require `lkjmc.user.party` | `menu.player-picker.empty` | `/party invite <player>` |
| `settings` | Direct menu daemon commands | `player.settings.toggle` payloads | User setting permissions are enforced by daemon/command path | Disabled only on command failure | Toggle HUD and hotbar token |
| `language` | Direct menu daemon commands | `player.settings.set` payloads | User language permission is enforced by daemon/command path | Disabled only on command failure | Set language to `en` or `ja` |

## Diagnostics

All loader failures map to localized diagnostics for missing daemon config, token
missing, token unreadable, HTTP failure, auth failure, unknown command, command
failure, database not configured, database unavailable, schema mismatch, or
permission denial. Diagnostics never print tokens, secret URLs, raw stack traces,
or generated secrets.
