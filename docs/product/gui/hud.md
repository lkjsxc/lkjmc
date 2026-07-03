# HUD setting

## Purpose

HUD is the player preference that enables or disables passive status frames sent
through the action-bar channel.


## Status

implemented

## Behavior

`/hud on` enables passive action-bar frames. `/hud off` stops passive frames.
The setting is stored durably through `player.settings.hud` or
`player.settings.toggle` and is read by the action-bar snapshot path.

HUD off does not block deliberate event-result frames such as purchase,
exchange, random teleport, or reward feedback when those events intentionally use
the action bar.

## Snapshot data

The daemon action-bar snapshot returns HUD enabled state, playtime seconds,
point balance, current server id, server player count, network online count,
daily availability, random teleport cooldown, and optional adventure or transfer
status.
