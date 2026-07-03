# Action bar

## Purpose

The action bar is the Minecraft display channel for short transient frames. It is
not the HUD setting itself.


## Status

implemented

## Current status

A shared reducer deduplicates repeated frames. Paper sends event frames for
short results and passive status frames when the player HUD setting is enabled.
The daemon provides an action-bar snapshot containing playtime, points, server,
online counts, daily availability, and random teleport cooldown.

## Passive frames

Passive frames may show playtime, point balance, current server, online count,
daily availability, random teleport cooldown, adventure status, and transfer
status. Frames are compact and never contain secrets, raw JSON, or stack traces.
Playtime uses minutes and hours only; the largest unit is `h`.

## Priority

1. Critical admin or daemon diagnostic.
2. Teleport, purchase, exchange, or reward result.
3. Adventure, temporary instance, or transfer countdown.
4. Claim protection denial or confirmation.
5. Daily reward availability.
6. Passive playtime, points, server, and online count.

## Rules

A pure reducer chooses the highest-priority unexpired frame. Passive frames send
only when HUD is enabled and the frame changes or the refresh interval expires.
Explicit event-result frames may still use the action-bar channel even when HUD
is disabled because HUD controls passive status, not the channel.
