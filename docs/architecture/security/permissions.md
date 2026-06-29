# Permissions

## Purpose

This document defines the current Minecraft permission contract.

## User nodes

- `lkjmc.user.menu` — open `/menu`; Paper default true.
- `lkjmc.user.language` — use `/lang`; Paper default true.
- `lkjmc.user.home` — use `/sethome` and `/home`; Paper default true.
- `lkjmc.user.warp` — use `/warp`; Paper default true.
- `lkjmc.user.teleport.request` — use `/tpa` and `/tpaccept`; Paper default true.
- `lkjmc.user.points` — use `/points`; Paper default true.
- `lkjmc.user.exchange` — use `/exchange`; Paper default true.
- `lkjmc.user.party` — use `/party`; Paper default true.
- `lkjmc.user.achievements` — use `/achievements`; Paper default true.
- `lkjmc.user.hud` — use `/hud`; Paper default true.
- `lkjmc.user.shop` — use `/shop` and `/buy`; Paper default true.
- `lkjmc.user.kit` — use `/kit`; Paper default true.
- `lkjmc.user.vote` — use `/vote`; Paper default true.
- `lkjmc.user.mail` — use `/mail`; Paper default true.
- `lkjmc.user.report` — use `/report`; Paper default true.
- `lkjmc.user.claim` — use `/claim`; Paper default true.
- `lkjmc.user.daily` — use `/daily`; Paper default true.
- `lkjmc.user.adventure` — use `/endexpedition`; Paper default true.

## Admin nodes

- `lkjmc.admin.status` — use `/lkjmc status`; Paper default op.
- `lkjmc.admin.admin` — manage lkjmc admin roles and grants.
- `lkjmc.admin.economy` — manage economy rates and default catalog seeding.
- `lkjmc.admin.reload` — use Velocity reload and restart warning commands.
- `lkjmc.admin.warp` — use `/setwarp`; Paper default op.
- `lkjmc.admin.send` — use Velocity `/lkjmc send`.
- `lkjmc.admin.instance.list` — list managed instances.
- `lkjmc.admin.instance.create` — create managed instances.
- `lkjmc.admin.instance.start` — start managed instances.
- `lkjmc.admin.instance.stop` — stop managed instances.
- `lkjmc.admin.instance.restart` — restart managed instances.
- `lkjmc.admin.instance.delete` — delete managed instances.
- `lkjmc.admin.reports` — use `/reports`; Paper default op.
- `lkjmc.admin.warn` — use warning and note commands; Paper default op.
- `lkjmc.admin.ban` — use ban commands; Paper default op.
- `lkjmc.admin.mute` — use mute commands; Paper default op.
- `lkjmc.admin.announce` — use `/announce`; Paper default op.
- `lkjmc.admin.claim` — override chunk claim protection; Paper default op.

## Source owners

- Java constants: `platforms/jvm/common/src/main/java/com/lkjmc/common/permission/PermissionNodes.java`.
- Paper command metadata: `platforms/jvm/paper/src/main/resources/plugin.yml`.
- Minecraft command mapping: [../../product/commands/minecraft.md](../../product/commands/minecraft.md).

## Verification

`scripts/check-permissions.py` checks that this document mentions every current
Java permission constant and every permission declared in Paper metadata, and
that this document does not mention permission names outside those source owners.
