# Claims

## Purpose

This area owns the player-facing chunk claim product contract.

## Table of contents

- [Commands](commands.md)
- [Protection](protection.md)

## Current status

Claims are the next gameplay domain and are not implemented yet. Do not register
`/claim`, daemon claim commands, CLI claim commands, or smoke checks until the
schema, daemon behavior, cache, command adapter, and protection listener are
real.

## Product rules

- Claims are chunk-based for the first slice.
- A claim belongs to one player UUID and display name.
- Claim names are unique per owner among active claims.
- A chunk can belong to at most one active claim per instance and world.
- Trusted players can build and interact in trusted claims.
- Operators with `lkjmc.admin.claim` can inspect and override after the node
  exists in code.
