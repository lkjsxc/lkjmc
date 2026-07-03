# Admin menus

## Purpose

This document owns the operator inventory menus opened from the root Admin slot.


## Status

implemented

## Routes

- `admin`: dashboard and health summary.
- `admin-servers`: live server list plus Create Server entry.
- `admin-server-detail`: selected server summary and operations.
- `admin-server-stop-confirm`: selected server stop confirmation.
- `admin-server-restart-confirm`: selected server restart confirmation.
- `admin-server-delete-confirm`: selected server delete confirmation.
- `admin-server-create-*`: kind, template, jar, option, EULA, and final create
  confirmation steps.
- `admin-config`: doctor, reload, and restart warning entry points.
- `admin-security`: daemon token and grant management entry points.
- `admin-economy`: catalog seeding and shop administration entry points.
- `admin-moderation`: reports and player moderation entry points.
- `admin-audit`: recent privileged audit entry points.
- `admin-web`: authenticated web-control guidance.

## Rules

Every enabled row must dispatch to a real command or daemon mutation. Rows that
lack permission, data, daemon connectivity, or implementation render disabled
localized copy. The server menu is list-first: selecting a server opens operations
for that exact server id. Destructive operations use confirmation routes with
that id and Back as cancel. Text input is reserved for free-form reasons or names
when no picker or confirmation route can provide exact context.

## Root entry

The root menu reserves slot `31` for Admin. Documentation lives in slot `30` and
must never collide with Admin or Staff tools.

## Permission mapping

- Health rows use `lkjmc.admin.status`.
- Config reload rows use `lkjmc.admin.reload`.
- Grant, revoke, role, audit, and token rows use `lkjmc.admin.admin`.
- Server lifecycle rows use the `lkjmc.admin.instance.*` nodes and carry exact
  route params or payload fields instead of asking operators to retype ids.
- Economy rows use `lkjmc.admin.economy`.
- Announcements use `lkjmc.admin.announce`.
- Moderation rows use report, warn, ban, mute, claim, and warp admin nodes.

Daemon authorization remains final even when adapter visibility caches are stale.
