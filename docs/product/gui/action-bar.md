# Action bar

## Purpose

The action bar is the Minecraft channel for compact continuous status and short
priority result frames. It is not a sidebar, bossbar, title, or chat log.

## Status

implemented

## Runtime contract

Paper keeps separate cached snapshot and render loops. The frame renders about
every four ticks for players with passive status enabled. Daemon snapshots
refresh on a slower cadence, never on every frame, and all daemon work stays off
scheduler threads.

Rendering uses the shared MiniMessage text helper so catalog keys, item text,
chat feedback, and action-bar frames use one parsing path. If remote data is
stale or unavailable, the renderer keeps local playtime, local server id, and
known online counts visible, omits unknown fields, and never invents points.

## Passive content

Passive frames may include playtime, points when known, current server, local
server online count, network online count, daily reward state, random-teleport
cooldown, active adventure state, and wake-and-join or transfer state. Playtime
uses minutes and hours only; the largest unit is `h`.

## Priority order

1. Critical action result, refund, or safe daemon diagnostic.
2. Transfer, wake-and-join ready, or adventure state.
3. Paid random-teleport reservation, cooldown, or refund.
4. Teleport, purchase, exchange, kit, daily, vote, or achievement result.
5. Daily reward ready.
6. Passive status.

Priority frames expire quickly, then passive status resumes without duplicate
suppression that would hide required refreshes.

## Setting semantics

Player copy says Action Bar. `/hud` may remain a command alias only when command
docs identify HUD as the durable preference name for passive action-bar frames.
The setting does not block deliberate event-result frames.

## Verification

Pure formatter tests cover minute and hour units. Reducer tests cover priority,
expiry, stale daemon data, passive refresh, and MiniMessage rendering through
catalog keys.
