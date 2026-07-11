# Permission coverage

## Purpose

This document maps permission source owners to checked documentation.

## Source owners

- Java constants: `platforms/jvm/common/src/main/java/com/lkjmc/common/permission/PermissionNodes.java`.
- Paper metadata: `platforms/jvm/paper/src/main/resources/plugin.yml`.

## Checked docs

- Permission contract: [../architecture/security/permissions.md](../architecture/security/permissions.md).
- Minecraft command mapping: [../product/commands/minecraft.md](../product/commands/minecraft.md).

## Identity and proof boundary

Permission names are adapter-visible capability labels, not authenticated
identity. The command envelope actor and caller-provided `platformPermission`
value are untrusted; `crates/lkjmc-daemon/src/authz.rs` makes authorization
from an authenticated transport subject and, where available, durable grants.
A root token subject is broad daemon access, not a Minecraft-player identity.

`check-permissions.py` deterministically proves only name parity among Java
constants, Paper metadata, and the permission owner document. It does not prove
transport authentication, durable-grant lookup, adapter enforcement, or a live
command result. Those require their owner tests and, when applicable, Compose
or live evidence.

A permission name belongs in Java common before adapters rely on it. Paper
metadata must declare defaults for Paper-registered commands.
