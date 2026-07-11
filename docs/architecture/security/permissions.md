# Permissions

## Purpose

This document defines the shipped local Minecraft permission contract.

## Status

implemented

## User nodes

- `lkjmc.user.menu` — open `/menu`; Paper default true.
- `lkjmc.user.docs` — open `/docs`; Paper default true.

No admin permission or daemon command permission is registered by a Java plugin.

## Authorization provenance

The command-envelope actor, principal fields, platform permission, `op`, and
cached grant values are not authorization proof. Root tokens are CLI-shaped
operator credentials. Paper/Folia and Velocity daemon adapters are withdrawn
pending trusted identity/session attestation and cannot obtain authority from a
token file or a plugin permission.

## Source owners

- Paper command metadata: `platforms/jvm/paper/src/main/resources/plugin.yml`.
- Minecraft command mapping: [../../product/commands/minecraft.md](../../product/commands/minecraft.md).

## Verification

`scripts/check-permissions.py` checks the two Paper metadata permissions and
this owner document. The Java containment checker proves no daemon permission
resolver or admin registration is packaged.
