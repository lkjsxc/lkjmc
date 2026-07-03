# Dynamic menus

## Purpose

This document owns daemon-backed dynamic inventory surfaces.


## Status

implemented

## Current status

The menu routes described here exist in source. The current playable smoke
proves `/menu`, server-list loading, daemon-backed player menus, typed safe
states, settings actions, and no unintended inventory close on covered routes.

## Data policy

Dynamic menus render live daemon data when the daemon exposes a real command and
typed adapter. Missing data first renders loading and then real data, a true
empty state, a permission state, or a typed diagnostic. Loading, diagnostic, and
loaded replacements preserve the current route stack and session. Playable smoke
must prove `/menu`, server-list loading, daemon-backed player menus, typed
failures, and no ordinary-click close before this surface is marked healthy.

## Server surface

The public servers menu uses `instance.list`, desired state, observed process
state, and presence once available. The Admin server surface is list-first: it
shows live instances before operations, opens a selected-server detail route, and
uses confirmation routes for stop, restart, and delete. A stopped server can
start when the shared admin permission resolver allows
`lkjmc.admin.instance.start`. A suspended public target can show wake-and-join
controls when daemon queue, expiry, cancellation, permission, and Velocity
transfer checks are available. A running server can stop only when the resolver
allows `lkjmc.admin.instance.stop` and the presence count is zero unless a
documented force path is confirmed. Starting, pending, cancelled, expired,
occupied, denied, restart, delete, and transfer controls show exact disabled
reasons instead of fake actions.

## Travel and claims

Travel uses homes, warps, and teleport request daemon data. Homes and warps must
render daemon-backed lists before direct teleport controls appear; selecting a
listed home or warp may use command parity only because the item payload supplies
the exact safe command target. The Homes route includes Set Home Here in slot
`45`, generates `home`, then `home-2`, and opens a confirmation route that sends
`serverId` plus a nested Paper `location` object. Custom home names are future
advanced-only behavior, not the ordinary create path. Homes, warps, teleports,
and player pickers use slot `49` as true Back so Travel and child lists cannot
loop. Random Teleport loads a daemon quote before confirmation and never charges
when safe-location search fails. Claims use claim list and current-chunk inspection. Claim creation uses a
generated name, opens a confirmation route, and then runs the real `/claim
create <name>` command path with that generated name. Claim deletion uses a
confirmation detail route that preserves the exact claim name. Trust controls use
an online-player
picker and preserve the exact claim name. Actions that require a player target
use a picker or a command parity item only when the context is real.

## Economy and social

Economy uses points, shop, adventure catalog, kits, votes, and daily reward
data. The live shop slice renders daemon shop items with balance, categories,
prices, affordability, delivery executor, and exact disabled reasons. Purchase
controls are enabled only when item metadata declares a supported delivery
executor such as `minecraft-item` or `adventure` and the player can afford it;
unsupported or unaffordable items stay disabled. The live kit slice renders daemon kit definitions and can claim a
selected kit because the command payload supplies the exact kit id. The live
daily slice renders daemon claim status and only enables the claim command when
today's reward is unclaimed. The live vote slice renders daemon vote links and
runs a selected-link `/vote <id>` command that sends the exact URL for copying.
Shop detail uses direct purchase controls only for real executor paths. Social
uses party, mail, and reports data. Party renders current daemon party status;
leave uses a confirmation route when a party exists, invite uses an
online-player picker, and create calls `player.party.create` without text input
so the daemon can generate `<player>'s Party`, then a duplicate-free suffix. Mail
inbox entries can read a selected daemon message by id. Reports render only when
the shared admin permission resolver allows `lkjmc.admin.reports`; report detail
rows can resolve or dismiss after a confirmation route preserves the exact id.
Text-entry flows are not faked in inventory.

## Profile and settings

Profile and settings use language, HUD, hotbar token preference, points, and
achievement summaries. The live profile slice renders point balance and
achievement counts from daemon data; the achievements route shows locked,
in-progress, claimable, and claimed states. Claimable achievements expose real
reward-claim actions and disabled reasons when an executor is unavailable.
Language selection and HUD or hotbar token toggles send daemon-backed player
settings requests asynchronously, return to the player scheduler, update cached
locale or token state, and refresh the current route after completion. Persisted
language beats platform locale for menus, commands, docs, and action-bar text.
Empty configured-data lists use empty rows, not daemon failure copy.

## Diagnostic classes

Menu diagnostics distinguish daemon not configured, token missing, token file
unreadable, HTTP failure, auth failure, command unknown, command failed,
database not configured, database unavailable, schema mismatch, and permission
denial. Diagnostics include a safe operator hint and never print tokens, secret
URLs, raw stack traces, or generated secrets.
