# Contract checks

## Purpose

This document defines automated checks that keep docs and source-owned contracts
aligned.

## Checks

- `scripts/check-command-docs.py` compares daemon command literals, CLI command
  families, Paper command metadata, Paper permissions, and Velocity root command
  registrations with owner docs.
- `scripts/check-permissions.py` compares `PermissionNodes.java`, Paper
  `plugin.yml`, and [../architecture/security/permissions.md](../architecture/security/permissions.md).
- `scripts/check-locales.py` compares English and Japanese catalog leaf keys in
  repository config and JVM resources.

## Scope boundaries

The checks use simple parsing and stable source-owner files. They intentionally
avoid proving deep semantic behavior. Runtime behavior is covered by Rust, JVM,
smoke, and Compose verification gates.

## Rule

When a relationship is intentionally not checked, document the reason here
before relying on it.
