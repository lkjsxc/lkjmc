# Minecraft commands

## Purpose

This document defines the target in-game command surface.

## Velocity commands

- `/lkjmc`
- `/lkjmc status`
- `/lkjmc server list`
- `/lkjmc server start <server>`
- `/lkjmc server stop <server>`
- `/lkjmc server restart <server>`
- `/lkjmc server create <server> <template>`
- `/lkjmc server delete <server>`
- `/lkjmc send <player> <server>` (implemented as a real proxy transfer when
  target player and server are registered)
- `/lkjmc reload`
- `/hub`

## Paper and Folia commands

Implemented:

- `/lkjmc status`
- `/lkjmc server list|start|stop|restart|create|delete` calls the daemon when
  configured.
- `/menu`
- `/lang <en|ja>` persists the player's language through the daemon when
  configured.
- `/points` reads the PostgreSQL-backed points balance through the daemon when
  configured.
- `/sethome <name>` stores the player's current server-local location.
- `/home <name>` teleports to a stored home, requesting a profile-safe proxy
  transfer first when the home is on another server.
- `/setwarp <name>` stores an operator warp on the current server instance.
- `/warp <name>` teleports to a stored warp, requesting a profile-safe proxy
  transfer first when the warp is on another server.
- `/tpa <player>` and `/tpaccept <player>` handle same-server requests and
  profile-safe cross-server requests through the proxy bridge.
- `/party create <name>`, `/party invite <player>`, `/party accept <player>`,
  `/party info`, and `/party leave` manage PostgreSQL-backed parties.
- `/achievements` lists PostgreSQL-backed claimed achievements. First join,
  `/sethome`, and successful `/buy` grant built-in achievements when daemon HTTP
  is configured.
- `/hud <on|off>` persists the player's HUD preference, shows an immediate
  localized preview, and controls the periodic action-bar HUD refresh.
- `/shop` lists configured PostgreSQL-backed shop items and `/buy <item>`
  purchases an item with points when enough balance exists.
- `/kit` lists PostgreSQL-backed points kits and `/kit claim <kit>` grants the
  configured kit reward when its cooldown allows.
- `/vote` lists PostgreSQL-backed server voting links.
- `/mail inbox`, `/mail read <id>`, and `/mail send <player> <message>` manage
  PostgreSQL-backed player mail for players known to the network.
- `/report <player> <reason>` records a PostgreSQL-backed moderation report
  without blocking scheduler threads.
- `/reports` lists open PostgreSQL-backed moderation reports for operators.
- `/reports resolve <id>` and `/reports dismiss <id>` close moderation reports.
- `/warn <player> <reason>` records a PostgreSQL-backed player warning.
- `/warnings <player>` lists PostgreSQL-backed warnings for operators.
- `/note <player> <note>` records a PostgreSQL-backed moderation note.
- `/notes <player>` lists PostgreSQL-backed notes for operators.
- `/ban <player> <reason>` and `/unban <player>` write PostgreSQL-backed
  moderation state; Velocity denies banned players during login.
- `/mute <player> <reason>` and `/unmute <player>` manage PostgreSQL-backed
  chat mutes that Paper checks before chat messages are accepted.
- `/daily` grants a PostgreSQL-backed once-per-day points reward.
- `/announce <message>` records and broadcasts a PostgreSQL-backed server announcement.

Target commands not implemented yet: `/mute <player> <reason>` and
`/unmute <player>` should enforce PostgreSQL-backed chat mutes.
