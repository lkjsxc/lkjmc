# Temporary adventure and wake queue

## Purpose

Sequence the next gameplay work after bootstrap truthfulness.

## Temporary instances

PostgreSQL tables and typed store helpers exist for temporary instance ownership,
generated world paths, visibility, retention, lifecycle state, and audit
correlation, a pure allocation planner, daemon lifecycle commands, cleanup
worker, Velocity registration hints, transfer intents, and End Expedition
purchase/start/refund transaction, `/endexpedition` solo/party, transfer
intents, and confirmation menu buttons exist for hidden world paths, ports,
readiness, transfer, return command, automatic pre-expiry return, stop, and
explicit cleanup. Wake-and-join queueing for suspended backends now has store,
daemon, and Velocity admin command coverage.

## End Expedition

Create the purchase flow only after temporary instance creation is real. Points
deduction, adventure session creation, and instance creation must commit in one
transaction. Startup or readiness failure refunds through the points ledger and
marks the session failed.

## Wake-and-join

`instance.wake.request` enqueues player intent, starts or verifies the backend,
marks the queue ready or failed, and returns a target server. Velocity admin
`/lkjmc wake send` consumes that daemon path before profile-safe transfer. Public
menu controls use durable request, status, cancellation, expiry cleanup, consume,
and transfer safety paths; unavailable states render disabled reasons.

## Verification

Add deterministic store and planner tests first, daemon lifecycle tests second,
and opt-in live Velocity/Folia smokes only after the daemon path works.
