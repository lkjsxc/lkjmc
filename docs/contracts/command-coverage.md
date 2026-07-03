# Command coverage

## Purpose

This document redirects daemon command coverage to the structural registry.

## Status

Daemon command coverage moved to [command-registry.md](command-registry.md) and
`contracts/commands.json`.

## Rule

Do not register commands in code until the behavior is real and the owner docs
name the command, permission, localization, completion behavior, and verification
path. `/lkjmc` command execution and suggestions must be generated from the
shared model rather than duplicated per adapter.
