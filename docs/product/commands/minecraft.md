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
- `/home <name>` teleports to a stored home on the same server instance.
- `/setwarp <name>` stores an operator warp on the current server instance.
- `/warp <name>` teleports to a stored warp on the same server instance.
- `/tpa <player>` and `/tpaccept <player>` handle same-server teleport
  requests.
- `/party create <name>`, `/party invite <player>`, `/party accept <player>`,
  `/party info`, and `/party leave` manage PostgreSQL-backed parties.
- `/achievements` lists PostgreSQL-backed claimed achievements. First join
  grants the built-in first-login achievement when daemon HTTP is configured.

Target commands not implemented yet: cross-server homes/warps, cross-server
teleport, shop, and broader achievement progress triggers.
