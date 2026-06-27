# Dynamic menus

## Purpose

This document owns daemon-backed dynamic inventory surfaces.

## Data policy

Dynamic menus render live daemon data when the daemon exposes a real command and
typed adapter. Missing data first renders loading and then an explicit
unavailable or disabled state; it must not render fake actions.

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
the exact target. Claims use claim list and current-chunk inspection. The live
claims slice renders owned claim summaries from `claim.list`, renders
current-chunk creation as disabled until a name input flow exists, and confirms
claim deletion from a detail route that preserves the exact claim name. Trust
controls stay disabled until picker flows exist. Actions that require a player target use a picker or a command parity item only
when the context is real. Teleport request menus expose the existing accept
command but keep new request controls disabled until a player picker exists.

## Economy and social

Economy uses points, shop, kits, votes, and daily reward data. The live shop
slice renders daemon shop items and prices. Purchase controls are enabled only
when item metadata declares a supported `minecraft-item` delivery executor;
items without supported delivery metadata stay disabled. The live kit slice
renders daemon kit definitions and can claim a selected kit because the command
payload supplies the exact kit id. The live daily slice renders daemon claim
status and only enables the claim command when today's reward is unclaimed. The
live vote slice renders daemon vote links
as copy-only disabled entries until a platform-safe open/copy capability exists.
Shop detail uses direct purchase controls only for real executor paths. Social
uses party, mail, and reports data. Party renders current daemon party status;
leave uses a confirmation route when a party exists, while create and invite
stay disabled until input and picker flows exist. Mail inbox entries can read a selected daemon message
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
after completion.
