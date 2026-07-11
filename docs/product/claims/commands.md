# Claim commands

## Purpose

This document defines daemon and CLI claim operations and the withdrawn Java
claim command boundary.

## Status

implemented

## Java boundary

Paper/Folia `/claim` and its adapter are withdrawn pending trusted
identity/session attestation. No Paper permission, local snapshot, or command
maps a player request to a daemon mutation.

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

Mutating commands are atomic: claim state, related progression, and audit rows
commit together or not at all. Claim creation accepts first-contact players
before a prior profile session.

## Localization

Claim locale data is daemon/store copy only while Java claim feedback is
withdrawn.
