# Temporary adventure and wake queue

## Purpose

Sequence the next gameplay work after bootstrap truthfulness.

## Temporary instances

PostgreSQL tables and typed store helpers exist for temporary instance ownership,
generated world paths, visibility, retention, lifecycle state, and audit
correlation. The next slice is pure runtime allocation and daemon lifecycle
handlers for generated worlds, readiness, stop, and cleanup.

## End Expedition

Create the purchase flow only after temporary instance creation is real. Points
deduction, adventure session creation, and instance creation must commit in one
transaction. Startup or readiness failure refunds through the points ledger and
marks the session failed.

## Wake-and-join

Transfers to suspended backends stay disabled until a daemon queue can enqueue
player intent, start the backend, wait for readiness, retry Velocity transfer,
expire with localized failure, and clear state on success or cancellation.

## Verification

Add deterministic store and planner tests first, daemon lifecycle tests second,
and opt-in live Velocity/Folia smokes only after the daemon path works.
