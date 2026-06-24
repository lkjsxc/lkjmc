# Architecture overview

## Purpose

This document defines the target component graph for `lkjmc`.

## Flow

```text
Minecraft clients
  -> Velocity proxy with lkjmc-velocity plugin
    -> Paper/Folia server with lkjmc-paper plugin
    -> Vanilla/custom/modded server without plugin support

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
daemon. The daemon owns OS processes and jar files.
