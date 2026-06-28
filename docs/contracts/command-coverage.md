# Command coverage

## Purpose

This document maps command source owners to checked documentation.

## Source owners

- Daemon command literals: `crates/lkjmc-daemon/src/*api*.rs` and dispatch
  modules named by the routers.
- CLI families and subcommands: `crates/lkjmc-cli/src/args*.rs`.
- Shared `/lkjmc` command model:
  `platforms/jvm/common/src/main/java/com/lkjmc/common/command/`.
- Paper command metadata: `platforms/jvm/paper/src/main/resources/plugin.yml`.
- Velocity root registrations: `VelocityCommands.java`.

## Checked docs

- Daemon: [../architecture/runtime/daemon/command-catalog.md](../architecture/runtime/daemon/command-catalog.md).
- CLI: [../product/commands/ssh-cli.md](../product/commands/ssh-cli.md).
- Minecraft: [../product/commands/minecraft.md](../product/commands/minecraft.md).

## Rule

Do not register commands in code until the behavior is real and the owner docs
name the command, permission, localization, completion behavior, and verification
path. `/lkjmc` command execution and suggestions must be generated from the
shared model rather than duplicated per adapter.
