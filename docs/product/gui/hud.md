# HUD setting

## Purpose

HUD is the durable internal setting that enables passive Action Bar status
frames.

## Status

planned

## Behavior

`/hud on` enables passive Action Bar frames. `/hud off` stops passive status
frames. The setting is stored durably through `player.settings.hud` or
`player.settings.toggle` and is read by the action-bar snapshot path.

HUD off does not block deliberate event-result frames such as purchase,
exchange, random teleport, reward feedback, transfer status, or safe daemon
diagnostics when those events intentionally use the action-bar channel.

## Text rendering

Player-facing labels, menu lore, and settings copy say Action Bar. The menu
engine renders the setting through locale keys and the shared MiniMessage helper.
The `/hud` command name remains compatibility only when command docs identify it
as the stored preference name.

## Snapshot data

The daemon action-bar snapshot returns the passive setting, playtime seconds,
point balance when available, current server id, server player count, network
online count, daily availability, random-teleport cooldown, and optional
adventure or transfer status.
