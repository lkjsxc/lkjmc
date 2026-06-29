# Dynamic menus

## Purpose

This document owns daemon-backed dynamic inventory surfaces.

## Field status

The menu routes described here exist in source, but field evidence shows
Minecraft inventory rows reporting daemon auth failure and many features are not
usable. Treat dynamic menus as an active blocker until playable smoke proves
server-list loading, one daemon-backed player menu, and no unintended close.

## Data policy

Dynamic menus render live daemon data when the daemon exposes a real command and
typed adapter. Missing data first renders loading and then real data, a true
empty state, a permission state, or a typed diagnostic. Loading, diagnostic, and
loaded replacements preserve the current route stack and session. Playable smoke
must prove `/menu`, server-list loading, daemon-backed player menus, typed
failures, and no ordinary-click close before this surface is marked healthy.

## Server surface

The servers menu uses `instance.list`, desired state, observed process state,
and presence once available. The live list renders real instances with stable
ordering. A stopped or suspended server can start from the row when the player
has `lkjmc.admin.instance.start`. A running server can stop from the row only
when the player has `lkjmc.admin.instance.stop` and the presence count is zero.
Starting, occupied, denied, restart, delete, and transfer controls show exact
disabled reasons instead of fake actions.

## Travel and claims

Travel uses homes, warps, and teleport request daemon data. Homes and warps must
render daemon-backed lists before direct teleport controls appear; selecting a
listed home or warp may use command parity only because the item payload supplies
the exact safe command target. Homes, warps, teleports, and player pickers use
slot `49` as true Back so Travel and child lists cannot loop. Claims use claim
list and
current-chunk inspection. The live claims slice renders owned claim summaries
from `claim.list`, prompts for a claim name through the next chat message before
running create; the prompt expires after 60 seconds. Claim deletion uses a
confirmation detail route that preserves the exact claim name. Trust controls use
an online-player picker and preserve the exact claim name. Actions that require
a player target use a picker or a command parity item only when the context is
real. Teleport request menus use an online-player picker for new requests and
expose the existing accept command.

## Economy and social

Economy uses points, shop, kits, votes, and daily reward data. The live shop
slice renders daemon shop items and prices. Purchase controls are enabled only
when item metadata declares a supported `minecraft-item` delivery executor;
items without supported delivery metadata stay disabled. The live kit slice
renders daemon kit definitions and can claim a selected kit because the command
payload supplies the exact kit id. The live daily slice renders daemon claim
status and only enables the claim command when today's reward is unclaimed. The
live vote slice renders daemon vote links and runs a selected-link `/vote <id>`
command that sends the exact URL for copying.
Shop detail uses direct purchase controls only for real executor paths. Social
uses party, mail, and reports data. Party renders current daemon party status;
leave uses a confirmation route when a party exists, invite uses an
online-player picker, and create prompts for a party name through the next chat
message with the same 60-second expiry. Mail inbox entries can read a selected daemon message
by id. Reports render only for `lkjmc.admin.reports`; report detail rows can
resolve or dismiss after a confirmation route preserves the exact report id.
Text-entry flows are not faked in inventory.

## Profile and settings

Profile and settings use language, HUD, hotbar token preference, points, and
achievement summaries. The live profile slice renders point balance and claimed
achievement counts from daemon data; the achievements route renders claimed
achievements as informational rows. Language selection and HUD or hotbar token
toggles send daemon-backed player settings requests asynchronously, return to
the player scheduler, update cached token state, and refresh the current route
after completion. Empty configured-data lists use empty rows, not daemon failure
copy.

## Diagnostic classes

Menu diagnostics distinguish daemon not configured, token missing, token file
unreadable, HTTP failure, auth failure, command unknown, command failed,
database not configured, database unavailable, schema mismatch, and permission
denial. Diagnostics include a safe operator hint and never print tokens, secret
URLs, raw stack traces, or generated secrets.
