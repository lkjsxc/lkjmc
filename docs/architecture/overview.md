# Architecture overview

## Purpose

This document defines the implemented component graph for `lkjmc`.

## Flow

```text
Minecraft clients
  -> Velocity proxy with lkjmc-velocity plugin
    -> Paper/Folia server with lkjmc-paper plugin
    -> process-only server managed by the daemon

SSH / AI agent
  -> lkjmc CLI
    -> Unix socket JSON-RPC
      -> lkjmc-daemon
        -> PostgreSQL
        -> local process runtime
        -> jar registry
        -> templates
        -> logs
```

## Components

- `lkjmc-core`: pure Rust models, validation, and planners.
- `lkjmc-store`: PostgreSQL migrations and typed adapters.
- `lkjmc-daemon`: API, reconciliation, jars, templates, and processes.
- `lkjmc-cli`: SSH-friendly operator surface.
- `platforms/jvm/common`: Java records, i18n, menus, daemon client.
- `platforms/jvm/velocity`: proxy adapter.
- `platforms/jvm/paper`: Paper/Folia adapter.

## Dependency direction

Pure cores do not import adapters. Plugins request orchestration through the
daemon. The daemon owns OS processes and jar files. Durable product state flows
through PostgreSQL store helpers rather than plugin-local files.

## Navigation

- Runtime commands: [runtime/daemon/command-catalog.md](runtime/daemon/command-catalog.md).
- CLI commands: [../product/commands/ssh-cli.md](../product/commands/ssh-cli.md).
- Minecraft commands: [../product/commands/minecraft.md](../product/commands/minecraft.md).
- Permissions: [security/permissions.md](security/permissions.md).
