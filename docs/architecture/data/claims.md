# Claims data

## Purpose

This document defines the PostgreSQL model for chunk claims.

## Tables

- `player_claims`: claim identity, owner UUID, owner display name, active name,
  normalized `name_key`, creation time, and soft-delete time.
- `claim_chunks`: instance ID, world name, chunk X, and chunk Z attached to one
  claim.
- `claim_trusts`: trusted player UUID and display name for one claim.

## Constraints

- Active claim names are unique per owner by `owner_uuid` and `name_key`.
- A chunk can belong to at most one active claim per instance and world.
- Deleting a claim marks the claim deleted and removes active chunk and trust
  rows so the chunk can be claimed again.
- Store mutations that create, delete, trust, or untrust claims use typed
  `lkjmc-store` helpers.

## Source owners

- SQL migration: `migrations/021-claims.sql`.
- Pure Rust model: `crates/lkjmc-core/src/claim.rs`.
- Store helpers: `crates/lkjmc-store/src/claims.rs`.
- Daemon handlers: `crates/lkjmc-daemon/src/claim_*.rs`.
