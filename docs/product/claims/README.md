# Claims

## Purpose

This area owns the player-facing chunk claim product contract.


## Status

implemented

## Table of contents

- [Commands](commands.md)
- [Protection](protection.md)

## Current status

PostgreSQL-backed one-chunk claims are implemented for Paper/Folia. The daemon
owns durable claim state, Java common owns immutable snapshots and access
decisions, and the Paper adapter owns `/claim` plus event protection.

## Product rules

- Claims are chunk-based for the first slice.
- A claim belongs to one player UUID and display name.
- Claim names are unique per owner among active claims.
- A chunk can belong to at most one active claim per instance and world.
- Trusted players can build and interact in trusted claims.
- Operators with `lkjmc.admin.claim` can override protection.
- Unknown chunks are allowed when the daemon is unavailable; known chunks remain
  protected from the last snapshot.

## Outcome, journey, and evidence boundary

A player claims the current chunk, then manages that exact durable claim through
commands or menus while protection reads an asynchronous local snapshot. During
a daemon outage, the last known claims stay protected and unknown chunks remain
usable rather than globally locking play. Pure-policy, snapshot, and command
tests support this behavior; they do not establish freshness beyond the last
successful refresh on a live server.
