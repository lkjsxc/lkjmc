# Admin menus

## Purpose

This document owns the operator inventory menus opened from the root Admin slot.

## Routes

- `admin`: dashboard and health summary.
- `admin-servers`: server list and lifecycle entry points.
- `admin-config`: doctor, reload, and restart warning entry points.
- `admin-security`: daemon token and grant management entry points.
- `admin-economy`: catalog seeding and shop administration entry points.
- `admin-moderation`: reports and player moderation entry points.
- `admin-audit`: recent privileged audit entry points.
- `admin-web`: authenticated web-control guidance.

## Rules

Every enabled row must dispatch to a real command or daemon mutation. Rows that
lack permission, data, daemon connectivity, or implementation render disabled
localized copy. Destructive or reason-bearing operations use confirmation or text
input flows before daemon mutation.

## Root entry

The root menu reserves slot `31` for Admin. Documentation lives in slot `30` and
must never collide with Admin or Staff tools.

## Permission mapping

- Health rows use `lkjmc.admin.status`.
- Config reload rows use `lkjmc.admin.reload`.
- Grant, revoke, role, audit, and token rows use `lkjmc.admin.admin`.
- Server lifecycle rows use the `lkjmc.admin.instance.*` nodes.
- Economy rows use `lkjmc.admin.economy`.
- Announcements use `lkjmc.admin.announce`.
- Moderation rows use report, warn, ban, mute, claim, and warp admin nodes.

Daemon authorization remains final even when adapter visibility caches are stale.
