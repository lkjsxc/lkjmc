# Architecture overview

## Purpose

This document defines the implemented component graph for `lkjmc`.

## Status

implemented

## Flow

```text
Minecraft clients
  -> Velocity proxy with local presentation plugin
    -> Paper/Folia server with local documentation plugin

Discord users / operator browser / SSH
  -> command transport
    -> lkjmc-daemon
      -> PostgreSQL
      -> local process runtime or Kubernetes adapter
      -> jar registry, templates, and logs
```

## Current ownership

- `lkjmc-core` owns pure Rust models, validation, and planners.
- `lkjmc-store` owns PostgreSQL migrations and typed persistence adapters.
- `lkjmc-daemon` owns command dispatch, reconciliation, private web, and effects.
- `lkjmc-cli` and `lkjmc-discord` are command transports.
- JVM common provides local docs and presentation helpers; Paper/Folia and
  Velocity own only local platform callbacks.

## Effect boundary

Pure cores do not import effect adapters. Durable product state goes through
PostgreSQL store helpers. The daemon owns process, Kubernetes, asset, template,
and private-web effects. Java plugins do not request daemon commands or perform
product mutations while identity/session attestation is unavailable.

## Evidence boundary

Documentation checks establish topology only. Supported guarded checks require
external prerequisites; a missing prerequisite is a reported skip, not proof of
runtime health.

## Navigation

- Runtime commands: [runtime/daemon/commands](runtime/daemon/commands/README.md).
- CLI commands: [../product/commands/ssh-cli.md](../product/commands/ssh-cli.md).
- Minecraft commands: [../product/commands/minecraft.md](../product/commands/minecraft.md).
- Permissions: [security/permissions.md](security/permissions.md).
