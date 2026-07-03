# Claim protection

## Purpose

This task adds PostgreSQL-backed chunk claims as the next gameplay domain.


## Status

completed

## Contract

- Claims are chunk-based and belong to one owner UUID and display name.
- Claim names are unique per owner among active claims.
- One active chunk can belong to at most one claim per instance and world.
- Trusted players may build and interact in trusted claims.
- `lkjmc.admin.claim` can inspect and override.
- Paper/Folia protection reads an immutable in-memory snapshot and never calls
  the daemon from event threads.
- During daemon outage, known claimed chunks remain protected from the last
  snapshot and unknown chunks are allowed.

## Verification

Claims require docs, migration/store tests, daemon tests, Java common tests,
Paper command/listener tests where practical, and broad verification before
handoff.
