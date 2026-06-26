# Claim commands

## Purpose

This document defines the target claim command surface before implementation.

## Target Paper/Folia command

`/claim` must not be registered until every subcommand below calls real daemon
behavior and refreshes the local claim snapshot after successful mutations.

- `/claim create <name>` creates a one-chunk claim for the current chunk.
- `/claim list` lists active claims visible to the player.
- `/claim delete <name>` deletes an owned claim.
- `/claim trust <player>` trusts a player for the current or selected claim.
- `/claim untrust <player>` removes trust.
- `/claim here` shows the current chunk claim owner and trust state.

## Target daemon commands

Claim daemon commands will be documented in the daemon catalog only after they
are implemented. Planned names are `claim.create`, `claim.delete`,
`claim.trust`, `claim.untrust`, `claim.list`, and `claim.snapshot`.

## Localization

All player-facing claim messages must be added to English and Japanese catalogs
in the same change as the command implementation.
