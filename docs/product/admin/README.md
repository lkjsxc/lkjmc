# Admin

## Purpose

This area owns operator-facing admin behavior: roles, grants, visibility,
privileged command authorization, menus, and audit trails.

## Table of contents

- [Model](model.md)
- [Menus](menus.md)
- [Commands](commands.md)

## Current status

A role-to-permission catalog, durable grants, revoke/inspect helpers, admin
audit rows, daemon admin commands, CLI grant/revoke/inspect/audit commands,
daemon enforcement for documented admin command families, Minecraft command
visibility caches, and a first-class Admin menu entry are implemented.

## Contract

Admin behavior is product-owned rather than scattered through adapters. Platform
permissions and `op` are adapter inputs, but daemon grants are durable truth for
lkjmc admin roles. Paper, Velocity, web, CLI, and Discord adapters may use fresh
cached snapshots for visibility. Privileged daemon mutations authorize the
end-user or local operator principal and record safe audit rows.
