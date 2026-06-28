# End expedition

## Purpose

This target product contract describes a points-purchased pristine End challenge
implemented through temporary Folia instances.

## User flow

A player opens Economy or Temporary Adventures, selects End Expedition, reviews
cost, party size, time limit, loot rules, and risk rules, then confirms. The
daemon validates points, deducts points, creates an adventure session, creates a
temporary Folia instance, starts it, registers it through Velocity, and
transfers participants when ready.

## Data contract

The daemon records adventure session id, buyer, participants, point ledger entry,
temporary instance id, state, start deadline, hard stop deadline, refund state,
and audit ids. The daemon purchase command spends points, creates the adventure
session, creates the temporary instance, and records the buyer participant in one
PostgreSQL transaction.

## Runtime rules

The backend is hidden from normal server lists, uses a unique port, generates a
fresh world directory, installs the `lkjmc` Paper plugin, configures Velocity
forwarding, and has aggressive autosuspend and cleanup policy.

## Failure behavior

If purchase validation, point spend, session creation, temporary instance
creation, or generated world creation fails, the command returns an error and no
points are deducted. If process start, readiness, registration, or first
transfer fails after points are deducted, the daemon refunds through the ledger,
marks the session failed, and audits the transition. Players see a localized
failure, not a live purchase success.

## Minecraft surfaces

`/endexpedition` is a live Paper/Folia command for solo starts, and
`/endexpedition party` includes the buyer's current party members as queued
participants. The command calls the daemon purchase flow, creates a short-lived
transfer intent for each local participant, then asks Velocity to perform the
profile-safe transfer. The Temporary Adventures menu has solo and party buttons
with confirmation routes that run the same commands.

## Current status

Adventure session and temporary instance tables, typed store helpers, explicit
daemon temporary instance runtime commands, Velocity registration hints, transfer
intents, cleanup worker, daemon purchase, startup, and refund on
startup/readiness failure, `/endexpedition`, party selection, confirmation
menu buttons, locale keys, and permission paths exist. Return-to-hub behavior is
not implemented yet.

## Current boundary

This is not a live shop item. The direct command, party variant, and menu
confirmation buttons are live; return-to-hub behavior remains disabled until
implemented end to end.
