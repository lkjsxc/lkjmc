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
and audit ids. Session and temporary instance creation share one PostgreSQL
transaction with the points deduction.

## Runtime rules

The backend is hidden from normal server lists, uses a unique port, generates a
fresh world directory, installs the `lkjmc` Paper plugin, configures Velocity
forwarding, and has aggressive autosuspend and cleanup policy.

## Failure behavior

If process start, readiness, registration, or first transfer fails after points
are deducted, the daemon refunds through the ledger, marks the session failed,
and audits the transition. Players see a localized failure, not a live purchase
success.

## Minecraft surfaces

Menus and commands may show disabled rows with exact reasons while the daemon
flow is absent. Live purchase buttons are registered only after daemon purchase,
startup, transfer, timeout, cleanup, refund, locale, and permission paths are
verified.

## Current boundary

This is not a live shop item yet. It may render only as a disabled item with an
exact inactive reason until purchase, instance creation, readiness, transfer,
stop, and cleanup are implemented end to end.
