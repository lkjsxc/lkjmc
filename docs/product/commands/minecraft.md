# Minecraft commands

## Purpose

This document defines the current in-game command surface and source owners.

## Velocity commands

- `/lkjmc` dispatches proxy admin subcommands in `VelocityCommands.java`.
- `/lkjmc status` reports proxy status and requires `lkjmc.admin.status`.
- `/lkjmc server list` lists registered servers and requires `lkjmc.admin.instance.list`.
- `/lkjmc server start <server>` calls `instance.start` and requires `lkjmc.admin.instance.start`.
- `/lkjmc server stop <server>` calls `instance.stop` and requires `lkjmc.admin.instance.stop`.
- `/lkjmc server restart <server>` calls `instance.restart` and requires `lkjmc.admin.instance.restart`.
- `/lkjmc server create <server> <template>` calls `instance.create` and requires `lkjmc.admin.instance.create`.
- `/lkjmc server delete <server> confirm` calls `instance.delete` and requires `lkjmc.admin.instance.delete`.
- `/lkjmc send <player> <server>` performs a profile-safe proxy transfer and requires `lkjmc.admin.send`.
- `/lkjmc reload` refreshes daemon-backed proxy registration and requires `lkjmc.admin.reload`.
- `/lkjmc restart warn <seconds>` broadcasts a warning and requires `lkjmc.admin.reload`.
- `/hub` sends the player to the registered `hub` server when available.

## Paper and Folia commands

- `/lkjmc status` and `/lkjmc server list|start|stop|restart|create|delete` are owned by `PaperCommands.java` and use admin instance permissions.
- `/menu` opens the localized menu and requires `lkjmc.user.menu`.
- `/lang <en|ja>` persists language and requires `lkjmc.user.language`.
- `/points` and `/points top` read point balances and require `lkjmc.user.points`.
- `/sethome <name>` and `/home <name>` manage homes and require `lkjmc.user.home`.
- `/setwarp <name>` requires `lkjmc.admin.warp`; `/warp <name>` requires `lkjmc.user.warp`.
- `/tpa <player>` and `/tpaccept <player>` require `lkjmc.user.teleport.request`.
- `/party create|invite|accept|info|leave` requires `lkjmc.user.party`.
- `/achievements` requires `lkjmc.user.achievements`.
- `/hud <on|off>` requires `lkjmc.user.hud`.
- `/shop` and `/buy <item>` require `lkjmc.user.shop`.
- `/kit [list|claim <kit>]` requires `lkjmc.user.kit`.
- `/vote` requires `lkjmc.user.vote`.
- `/mail inbox|read <id>|send <player> <message>` requires `lkjmc.user.mail`.
- `/report <player> <reason>` requires `lkjmc.user.report`.
- `/reports [resolve|dismiss <id>]` requires `lkjmc.admin.reports`.
- `/warn <player> <reason>` and `/warnings <player>` require `lkjmc.admin.warn`.
- `/note <player> <note>` and `/notes <player>` require `lkjmc.admin.warn`.
- `/ban <player> <reason>` and `/unban <player>` require `lkjmc.admin.ban`.
- `/mute <player> <reason>` and `/unmute <player>` require `lkjmc.admin.mute`.
- `/daily` requires `lkjmc.user.daily`.
- `/announce <message>` requires `lkjmc.admin.announce`.
- `/claim create|list|delete|trust|untrust|here` requires `lkjmc.user.claim`;
  protection override requires `lkjmc.admin.claim`.

## Registration source

Paper command names and metadata live in
`platforms/jvm/paper/src/main/resources/plugin.yml`. Executors are registered
in `LkjmcPaperPlugin.java`. Velocity registrations live in `VelocityCommands.java`.

## Verification

`scripts/check-command-docs.py` checks Paper command names, Paper permissions,
Velocity root command registrations, and CLI family docs.
