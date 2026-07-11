# Claims

## Purpose

This area owns the player-facing chunk claim product contract.


## Status

implemented

## Table of contents

- [Commands](commands.md)
- [Protection](protection.md)

## Current status

PostgreSQL-backed one-chunk claim storage is implemented. Paper/Folia `/claim`,
claim snapshots, and event protection are withdrawn pending trusted
identity/session attestation.

## Product rules

- Claims are chunk-based for the first slice.
- A claim belongs to one player UUID and display name.
- Claim names are unique per owner among active claims.
- A chunk can belong to at most one active claim per instance and world.
- Trust and override records are durable daemon data.
- No Java plugin currently applies these records to a live block event.

## Outcome, journey, and evidence boundary

Daemon and store tests support durable claim data. A Java player command, menu,
or live protection outcome is not shipped and must not be inferred from stored
records.
