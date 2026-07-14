# Permissions

## Purpose

This document defines the shipped local Minecraft permission contract.

## Status

implemented

## User nodes

- `lkjmc.user.menu` — open `/menu`; Paper default true.
- `lkjmc.user.docs` — open `/docs`; Paper default true.

No admin permission or daemon command permission is registered by a Java plugin.
The daemon-only `lkjmc.sync.read` credential scope authorizes revisioned read
transport; it is not a Minecraft permission and grants no command or mutation.

## Authorization provenance

The command-envelope actor, principal fields, platform permission, `op`, and
cached grant values are not authorization proof. Root tokens are CLI-shaped
operator credentials. Paper/Folia and Velocity mutation/application adapters are withdrawn pending
trusted identity/session attestation. Their read-only coordinator cannot obtain
player or operator authority from its token, cache, or plugin permission.

## Source owners

- Paper command metadata: `platforms/jvm/paper/src/main/resources/plugin.yml`.
- Minecraft command mapping: [../../product/commands/minecraft.md](../../product/commands/minecraft.md).

## Verification

`scripts/check-permissions.py` checks the two Paper metadata permissions and
this owner document. The Java containment checker proves no daemon permission
resolver or admin registration is packaged.
