# End expedition

## Purpose

This product contract describes the `end-expedition` catalog entry: a
points-purchased pristine End challenge implemented through temporary Folia
instances.


## Status

implemented

## User flow

A player opens Economy, Shop, or Temporary Adventures, selects End Expedition,
reviews cost, party size, time limit, loot rules, and risk rules, then confirms.
The daemon validates points, deducts points exactly once, creates an adventure
session, creates a temporary Folia instance, starts it, registers it through
Velocity, and transfers participants when ready.

## Data contract

The daemon records adventure id, session id, buyer, participants, point ledger
entry, temporary instance id, state, start deadline, hard stop deadline, refund
state, and audit ids. The generic daemon purchase command spends points, creates
the adventure session, creates the temporary instance, and records participants
in one PostgreSQL transaction.

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

`/endexpedition` is a convenience Paper/Folia command for solo starts, and
`/endexpedition party` includes the buyer's current party members as queued
participants. The command delegates to the generic `adventure.purchase` flow with
`adventureId=end-expedition`, creates a short-lived transfer intent for each
local participant, then asks Velocity to perform the profile-safe transfer.
`/endexpedition return` delegates to generic `adventure.return`, marks the
player left, and sends the player back to hub. Temporary End backends poll their
daemon lifetime and automatically run the same return flow for online players
shortly before expiry. The Temporary Adventures menu uses catalog rows, detail
pages, and confirmation routes for purchase actions.

## Current status

Adventure session and temporary instance tables, typed store helpers, explicit
daemon temporary instance runtime commands, Velocity registration hints, transfer
intents, cleanup worker, catalog purchase, startup, refund on startup/readiness
failure, `/endexpedition`, party selection, confirmation menu buttons,
return-to-hub command, automatic pre-expiry return, locale keys, and permission
paths exist.

## Shop delivery contract

Shop catalog item `adventure-end-expedition` uses the generic `adventure`
delivery executor with `adventureId=end-expedition`. The shop path must not run a
generic item purchase and an adventure purchase for one click. Unsupported
delivery metadata is rejected before any point deduction.

## Shop status

The direct command, party variant, return command, automatic pre-expiry return,
Temporary Adventures confirmation buttons, and shop catalog delivery executor
are live. The shop path delegates to the daemon adventure purchase flow and
records a shop purchase after successful adventure creation.
