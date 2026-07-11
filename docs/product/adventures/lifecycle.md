# Adventure lifecycle

## Purpose

This document owns purchase, transfer, return, failure, and cleanup behavior for
temporary generated-server adventures.


## Status

implemented

## Purchase flow

1. The adapter requests `adventure.catalog.list` or uses a cached catalog row.
2. The player confirms a selected adventure id and party option.
3. The daemon validates catalog, permission, points, party size, EULA acceptance,
   runtime facts, jar availability, and world allocation.
4. Point spend, instance row, temporary instance row, adventure session row, and
   participant rows commit in one PostgreSQL transaction.
5. The daemon starts the backend and readiness probe.
6. Transfer intents request participant transfer when the backend is ready; a
   plugin-message request is fire-and-forget and reports transfer pending, not
   completed purchase or confirmed arrival.

## Failure flow

Validation failures happen before points are deducted. Startup, readiness,
registration, or first-transfer failures after deduction grant an idempotent
refund with a deterministic correlation distinct from the session spend,
mark the session refunded, and audit the transition.

## Return flow

`adventure.return` validates the participant has an active session, marks the
participant left, and lets the adapter transfer the player back to hub. Backends
also run pre-expiry returns for online participants.

## Cleanup flow

The cleanup worker handles every shipped adventure with the temporary instance
cleanup policy. Deleted worlds and instance files must be owned by the temporary
session; a failed or interrupted attempt remains retryable. Cancellation stops
the actual backend before marking the session cancelled and uses the same refund
path when the session has not become active. An unhealthy or fenced recovered
runtime cannot prove backend absence, so cancellation fails without session,
instance, or refund persistence until an operator resolves the identity.
