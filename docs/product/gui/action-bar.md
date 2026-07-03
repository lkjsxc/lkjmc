# Action bar

## Purpose

The action bar is the Minecraft channel for continuous compact status and short
priority result frames. It is not a sidebar, bossbar, title, or chat log.

## Status

partial

Missing: Paper still needs a split cached snapshot loop and four-tick render
loop with tests proving daemon HTTP is not called every render tick.

## Runtime contract

Paper sends the current action-bar frame about every four ticks for players who
have passive status enabled. Daemon snapshots refresh on a slower cadence and
are cached per player. Rendering uses the best cached snapshot plus truthful
local session data.

Transient daemon failure must not make the action bar go silent. The renderer
keeps local playtime, local server id, and online counts visible, omits unknown
remote fields, and never invents a point balance.

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

Player-facing copy says Action Bar. `/hud` may remain as a command alias only if
command docs identify HUD as the stored preference for passive action-bar
frames. The setting does not block deliberate event-result frames.

## Verification

Pure formatter tests cover minute and hour units. Reducer tests cover priority,
expiry, stale daemon data, and passive refresh. Paper adapter tests use a fake
scheduler and fake daemon snapshot source.
