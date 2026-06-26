# Claims data

## Purpose

This document defines the target PostgreSQL model for chunk claims.

## Target tables

- `player_claims`: claim identity, owner UUID, owner display name, active name,
  normalized `name_key`, creation time, and soft-delete time.
- `claim_chunks`: instance ID, world name, chunk X, and chunk Z attached to one
  claim.
- `claim_trusts`: trusted player UUID and display name for one claim.

## Constraints

- Active claim names are unique per owner by `owner_uuid` and `name_key`.
- A chunk can belong to at most one active claim per instance and world.
- Deleting a claim removes its chunks and trust rows through foreign keys.
- Store mutations that change claims, chunks, or trust rows run in transactions
  when consistency requires it.

## Current status

Claims are not implemented yet. The next implementation slice should add a new
migration, pure Rust claim types in `lkjmc-core`, PostgreSQL helpers in
`lkjmc-store`, daemon commands, Java common cache records, and Paper/Folia
command and protection adapters.
