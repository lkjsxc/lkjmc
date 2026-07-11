# Architecture overview

## Purpose

This document defines the implemented component graph for `lkjmc`.


## Status

implemented

## Flow

```text
Minecraft clients
  -> Velocity proxy with lkjmc-velocity plugin
    -> Paper/Folia server with lkjmc-paper plugin
    -> process-only server managed by the daemon

Discord users
  -> lkjmc-discord interaction service
    -> daemon loopback HTTP commands

Operator browser
  -> authenticated /web pages on lkjmc-daemon

SSH / AI agent
  -> lkjmc CLI
    -> HTTP POST /command over Unix socket or optional TCP
      -> lkjmc-daemon
        -> PostgreSQL
        -> local process runtime or Kubernetes adapter
        -> jar registry
        -> templates
        -> logs
```

## Current ownership

- `lkjmc-core` owns pure Rust models, validation, and planners.
- `lkjmc-store` owns PostgreSQL migrations and typed persistence adapters.
- `lkjmc-daemon` owns command dispatch, reconciliation, private web, and effects.
- `lkjmc-cli` and `lkjmc-discord` are command transports.
- JVM common, Velocity, and Paper/Folia own platform-facing adapters only.

## Target dependency and effect boundary

Pure cores do not import effect adapters. Durable product state goes through
PostgreSQL store helpers, not plugin-local files. The daemon owns process,
Kubernetes, asset, template, and private-web effects; plugins, CLI, Discord,
and web request daemon commands rather than performing product mutations.

## Evidence and degraded behavior

The component roots named above are source evidence; command and runtime
contracts link from this page. Documentation checks establish link and status
shape only. Guarded Minecraft, Discord, and Kubernetes smokes require external
credentials or infrastructure; a missing prerequisite is a reported skip, not
proof that a transport, process, or cluster is healthy.

## Navigation

- Runtime commands: [runtime/daemon/commands](runtime/daemon/commands/README.md).
- CLI commands: [../product/commands/ssh-cli.md](../product/commands/ssh-cli.md).
- Minecraft commands: [../product/commands/minecraft.md](../product/commands/minecraft.md).
- Permissions: [security/permissions.md](security/permissions.md).
