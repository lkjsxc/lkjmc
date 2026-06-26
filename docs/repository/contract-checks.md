# Contract checks

## Purpose

This document defines deterministic repository checks that keep docs, source,
and generated metadata aligned.

## Current checks

- `scripts/check-command-docs.py` compares daemon command literals, CLI command
  families, Paper command metadata, Paper permissions, and Velocity root command
  registrations with owner docs.
- `scripts/check-permissions.py` compares `PermissionNodes.java`, Paper
  `plugin.yml`, and [../architecture/security/permissions.md](../architecture/security/permissions.md).
- `scripts/check-locales.py` compares English and Japanese catalog leaf keys in
  repository config and JVM resources.
- `scripts/check-docs.py` enforces doc README tables of contents, links, H1s,
  purpose headings, and banned release-label terms.
- `scripts/check-lines.py` enforces the 200-line file limit.

## Playable target checks

- `scripts/check-bootstrap-docs.py` verifies bootstrap command docs,
  quickstart EULA acceptance, playable Java and Bedrock ports, Velocity modern
  forwarding, and fallback `hub`.
- `scripts/check-asset-docs.py` verifies known plugin IDs, hash verification,
  ViaBackwards dependency on ViaVersion, and Geyser/Floodgate key handling.

## Scope boundaries

Contract checks are deterministic. Live downloads, Docker, and Minecraft server
launches belong in opt-in smoke checks unless a stable local cache makes them
repeatable.

## Rule

A check may fail on drift, but it must not create product state or print
secrets.
