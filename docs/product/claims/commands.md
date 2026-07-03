# Claim commands

## Purpose

This document defines the implemented claim command surface.

## Paper/Folia command

`/claim` is registered by `ClaimCommandAdapter` and requires
`lkjmc.user.claim` unless a server permission plugin overrides command defaults.

- `/claim create <name>` creates a one-chunk claim for the current chunk.
- `/claim list` lists the caller's active claims.
- `/claim delete <name>` deletes an owned claim.
- `/claim trust <player>` trusts an online player for the claim at the caller's
  current chunk.
- `/claim untrust <player>` removes trust for an online player at the caller's
  current chunk.
- `/claim here` reports the current chunk claim from the local snapshot.

## Operator CLI commands

- `lkjmc claim list --instance INSTANCE` lists active claim chunks for an
  instance.
- `lkjmc claim delete CLAIM_ID --yes` deletes a claim by ID with operator
  override.

## Daemon commands

- `claim.create`
- `claim.delete`
- `claim.trust`
- `claim.untrust`
- `claim.list`
- `claim.snapshot`

Mutating commands are atomic: claim state, related progression, and audit rows commit together or not at all. Claim creation accepts first-contact players before any prior profile session.

## Localization

Claim feedback uses English and Japanese `claim.*` locale keys in repository
config and JVM resources.
