# Command coverage

## Purpose

This document redirects daemon command coverage to the domain-sharded registry.

## Status

The contract source is `contracts/commands/README.json` and its listed shards.
It records only the 137 current daemon registrations at this revision. The
checked CLI and web consumers are source literals; an unlisted registration has
an explicit `internal` compatibility result.

## Rule

Do not register commands in code until behavior is real and its owner docs name
the command, permission, localization, completion boundary, and verification
path. Paper, Velocity, and Discord daemon execution remain withdrawn, so no
Minecraft, Discord, or generated binding is represented as covered.
