# Paper and Folia plugin

## Purpose

This document defines the target server plugin behavior.

## Responsibilities

- Provide a Folia-safe scheduler bridge.
- Capture and apply player profile snapshots.
- Provide inventory UI and localized player commands.
- Send server heartbeats.
- Run database and daemon operations asynchronously.

## Scheduler rules

Entity mutations run on player or entity schedulers. Region mutations run on
region schedulers. Database, filesystem, network, and process operations never
block scheduler threads.

## Current status

The Paper/Folia plugin jar registers `/lkjmc status`, `/menu`, `/lang <en|ja>`,
`/points`, `/sethome`, `/home`, `/setwarp`, `/warp`, `/tpa`, and `/tpaccept`,
creates a Folia-aware scheduler bridge, loads Java common localization
resources, opens localized inventory menus, sends daemon-backed instance
heartbeats when daemon HTTP and
`LKJMC_INSTANCE_ID` are configured,
loads and saves player profile snapshots through the daemon, reports daemon
status asynchronously from `/lkjmc status`, and cancels tracked scheduled work
on disable. Cross-server homes/warps/teleport, achievements, HUD, and
daemon-backed instance operations are exposed through `/lkjmc server ...` when
daemon HTTP is configured. Server-local parties can be created, invited,
accepted, inspected, and left through daemon-backed Paper commands. Claimed
achievements can be listed through the daemon, join/home/shop actions grant
built-in achievements, `/hud <on|off>` persists a HUD preference with an
immediate preview and periodic action-bar refresh, `/shop` plus `/buy <item>`
use daemon-backed points purchases, `/kit` lists and claims points kits,
`/daily` grants a daily points reward, `/mail` manages player mail, and moderation commands record, list, close
reports, record/list warnings, manage bans, and record/broadcast announcements
through the daemon. Cross-server homes and warps
request proxy transfers through the plugin-message bridge before teleporting on
arrival. Cross-server `/tpa` and `/tpaccept` use the same bridge to save the
source profile, transfer, and teleport after arrival.
