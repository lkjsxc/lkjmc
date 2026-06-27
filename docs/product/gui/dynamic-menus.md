# Dynamic menus

## Purpose

This document owns daemon-backed dynamic inventory surfaces.

## Data policy

Dynamic menus render live daemon data when the daemon exposes a real command and
typed adapter. Missing data renders loading, unavailable, or disabled states; it
must not render fake actions.

## Server surface

The servers menu uses `instance.list`, desired state, observed process state,
and presence once available. Start, stop, restart, and transfer controls render
only when the action path exists and permissions allow it. Stopped, starting,
full, hidden, or denied servers show exact disabled reasons.

## Travel and claims

Travel uses homes, warps, and teleport request daemon data. Claims use claim list
and current-chunk inspection. Actions that require a player target use a picker
or a command parity item only when the context is real.

## Economy and social

Economy uses points, shop, kits, votes, and daily reward data. Shop detail uses
direct purchase controls only for real executor paths. Social uses party, mail,
and reports data. Text-entry flows are not faked in inventory.

## Profile and settings

Profile and settings use language, HUD, hotbar token preference, points, and
achievement summaries. Language selection and HUD or hotbar token toggles are
the first required vertical slice: clicks send daemon-backed player settings
requests asynchronously, return to the player scheduler, update cached token
state, and refresh the current route after completion.
