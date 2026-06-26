# Permission coverage

## Purpose

This document maps permission source owners to checked documentation.

## Source owners

- Java constants: `platforms/jvm/common/src/main/java/com/lkjmc/common/permission/PermissionNodes.java`.
- Paper metadata: `platforms/jvm/paper/src/main/resources/plugin.yml`.

## Checked docs

- Permission contract: [../architecture/security/permissions.md](../architecture/security/permissions.md).
- Minecraft command mapping: [../product/commands/minecraft.md](../product/commands/minecraft.md).

## Rule

A permission name belongs in Java common before adapters rely on it. Paper
metadata must declare defaults for Paper-registered commands.
