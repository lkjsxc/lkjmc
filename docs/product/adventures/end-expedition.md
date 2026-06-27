# End expedition

## Purpose

This future product contract describes a points-purchased pristine End challenge
implemented through temporary Folia instances.

## User flow

A player opens Economy or Temporary Adventures, selects End Expedition, reviews
cost, party size, time limit, loot rules, and risk rules, then confirms. The
daemon validates points, deducts points, creates an adventure session, creates a
temporary Folia instance, starts it, registers it through Velocity, and transfers
participants when ready.

## Runtime rules

The temporary backend is hidden from normal server lists, uses a unique port,
generates a fresh world directory, installs the `lkjmc` Paper plugin, configures
Velocity forwarding, and has aggressive autosuspend and cleanup policy.

## Atomicity

Point deduction, session creation, and instance creation commit together. If
process start fails after deduction, the daemon refunds through the ledger,
marks the session failed, and audits every transition.

## Current boundary

This is not a live shop item yet. It may render only as a disabled future item
with an exact inactive reason until purchase, instance creation, readiness,
transfer, stop, and cleanup are implemented end to end.
