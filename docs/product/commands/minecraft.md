# Minecraft commands

## Purpose

This document defines the current in-game command surface and source owners.


## Status

implemented

Implemented: `/rtp` accepts overworld, Nether, and End profile grammar, paid
dimensions require confirmation, and player-facing passive status copy says
Action Bar.

## Current status

The shared command tree described here exists in source. The current playable
smoke proves `/lkjmc status`, `/lkjmc doctor`, `/lkjmc server`, server-list
output, root and server completion, daemon HTTP auth, and product-owned usage or
diagnostics through Minecraft.

## Public identity

`/lkjmc` is the only documented public control root for the network control
surface. Paper/Folia and Velocity may have adapter artifact names, but player
completion and product docs must not promote adapter-specific command families.

## Shared `/lkjmc` tree

The shared JVM command model owns path, permission, sender kind, usage,
execution target, and completion metadata. Paper/Folia and Velocity consume the
same tree where their capabilities overlap. Velocity must export `/lkjmc` as a
real Brigadier command graph built from this model so clients see the documented
syntax and suggestions before sending commands.

- `/lkjmc status` requires `lkjmc.admin.status`.
- `/lkjmc doctor` requires `lkjmc.admin.status` and reports command,
  daemon-HTTP, auth, database, and menu-contract health without secrets.
- `/lkjmc server list` requires `lkjmc.admin.instance.list`.
- `/lkjmc server start <server>` requires `lkjmc.admin.instance.start`.
- `/lkjmc server stop <server>` requires `lkjmc.admin.instance.stop`.
- `/lkjmc server restart <server>` requires `lkjmc.admin.instance.restart`.
- `/lkjmc server create <server> <template>` requires
  `lkjmc.admin.instance.create`; this Minecraft command omits the Minecraft
  EULA and an EULA-gated kind returns bodyless, non-retryable
  `adventure.confirmation_required`. No direct or admin command originates
  EULA consent.
- `/lkjmc server delete <server> confirm` requires
  `lkjmc.admin.instance.delete`.
- `/lkjmc reload` and `/lkjmc config reload` require `lkjmc.admin.reload`.
- `/lkjmc config check` requires `lkjmc.admin.status`.
- `/lkjmc admin role list`, `/lkjmc admin grant <principal> <role> --reason <reason>`,
  `/lkjmc admin revoke <principal> <role> --reason <reason>`,
  `/lkjmc admin inspect <principal>`, and `/lkjmc admin audit <lines>` require
  `lkjmc.admin.admin`.
- `/lkjmc security daemon-token status` and
  `/lkjmc security daemon-token rotate` require `lkjmc.admin.admin`.
- `/lkjmc economy seed-defaults` requires `lkjmc.admin.economy`.
- `/lkjmc adventure catalog`, `/lkjmc adventure start <adventure>`, and
  `/lkjmc adventure return` require `lkjmc.user.adventure`.
- `/lkjmc adventure sessions` requires `lkjmc.admin.instance.list`.
- `/lkjmc adventure cancel <session> <reason>` requires
  `lkjmc.admin.instance.delete`.
- `/lkjmc restart warn <seconds>` requires `lkjmc.admin.reload`.
- Velocity also exposes `/lkjmc send <player> <server>`,
  `/lkjmc temporary send <player> <instance>`, and
  `/lkjmc wake send <player> <server>` with `lkjmc.admin.send`.

Valid documented syntax, including root and intermediate prefixes such as
`/lkjmc server`, must return product output, product usage, no-permission copy,
or a safe daemon diagnostic. It must not leak parser-position internals.
Completion is permission-filtered and context-aware for subcommands, server ids,
player names, templates, roles, adventure ids, safe principal hints, line counts,
reasons, seconds, and `confirm`. Paper tab completion and Velocity Brigadier
visibility use the shared admin permission resolver with
platform permissions, `op`, and fresh cached durable grants. Stale or missing
grant snapshots may hide privileged completions, while daemon authorization
remains final. Server lifecycle output uses daemon instance state as product truth; proxy
registry entries may only add supplemental diagnostics. Inventory admin flows
must not ask operators to type a selected server id or a `confirm` token when the
route already carries that context.

## Paper and Folia player commands

- `/menu` opens the localized menu and requires `lkjmc.user.menu`.
- `/lang <en|ja>` persists language and requires `lkjmc.user.language`.
- `/points` and `/points top` read point balances and require
  `lkjmc.user.points`.
- `/sethome <name>` and `/home <name>` manage homes and require
  `lkjmc.user.home`.
- `/setwarp <name>` requires `lkjmc.admin.warp`; `/warp <name>` requires
  `lkjmc.user.warp`.
- `/tpa <player>` and `/tpaccept <player>` require
  `lkjmc.user.teleport.request`.
- `/rtp [overworld|nether|end] [confirm]` requires
  `lkjmc.user.random.teleport`. `/rtp` means free overworld teleport. Paid
  Nether and End profiles require fresh quote confirmation, charge only after
  safe-location search succeeds, and refund on final teleport failure.
- `/party create [name]|invite|accept|info|leave` requires `lkjmc.user.party`;
  omitted create names are generated by the daemon.
- `/achievements` requires `lkjmc.user.achievements` and must expose claimable
  reward state when the daemon supports reward claiming.
- `/hud <on|off>` requires `lkjmc.user.hud` and controls passive Action Bar
  status frames.
- `/shop` and `/buy <item>` require `lkjmc.user.shop`.
- `/exchange <material> <amount|all>` requires `lkjmc.user.exchange` and
  removes real inventory items before committing a daemon ledger grant.
- `/docs [search <query>|path]` requires `lkjmc.user.docs` and opens the
  in-game documentation browser.
- `/kit [list|claim <kit>]` requires `lkjmc.user.kit`.
- `/vote` requires `lkjmc.user.vote`.
- `/mail inbox|read <id>|send <player> <message>` requires `lkjmc.user.mail`.
- `/report <player> <reason>` requires `lkjmc.user.report`.
- `/reports [resolve|dismiss <id>]` requires `lkjmc.admin.reports`.
- `/warn <player> <reason>` and `/warnings <player>` require
  `lkjmc.admin.warn`.
- `/note <player> <note>` and `/notes <player>` require `lkjmc.admin.warn`.
- `/ban <player> <reason>` and `/unban <player>` require `lkjmc.admin.ban`.
- `/mute <player> <reason>` and `/unmute <player>` require `lkjmc.admin.mute`.
- `/daily` requires `lkjmc.user.daily`.
- `/endexpedition [party|return]` requires `lkjmc.user.adventure`; start paths
  forward unconfirmed requests and receive `adventure.confirmation_required`.
  Only the localized Temporary Adventures EULA-acceptance action originates
  consent; `return` is unaffected.
- `/announce <message>` requires `lkjmc.admin.announce`.
- `/claim create|list|delete|trust|untrust|here` requires `lkjmc.user.claim`;
  protection override requires `lkjmc.admin.claim`.

## Velocity utility command

`/hub` sends the player to the registered `hub` server when available.

## Registration source

Paper command names and metadata live in
`platforms/jvm/paper/src/main/resources/plugin.yml`. Executors and tab
completion are registered in `LkjmcPaperPlugin.java`. Velocity registrations
live in `VelocityCommands.java` and must use `BrigadierCommand` for `/lkjmc`.
The shared `/lkjmc` model lives in Java common.

## Verification

`scripts/check-command-docs.py` checks Paper command names, Paper permissions,
Velocity root command registrations, and CLI family docs. JVM tests must cover
shared parser, usage, permission filtering, and completion behavior before this
surface is considered healthy.
