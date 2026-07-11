# Temporary instances

## Purpose

This contract defines daemon-managed short-lived Minecraft backends.


## Status

implemented

## Data contract

Temporary instances are normal instance records plus durable temporary metadata:
owner command, visibility, generated world path, allocated ports, maximum
lifetime, autosuspend policy, retention policy, cleanup state, and audit
correlation. PostgreSQL is the source of truth; plugin memory is never the only
record of a temporary backend.

## Runtime contract

The daemon allocates unique ports, creates a generated world directory, renders
Folia-compatible config without the root daemon-token path, installs verified
required plugins, starts the process, waits for readiness, registers it through
Velocity, and hides it from normal server listings unless a product explicitly
exposes it. A temporary adapter remains daemon-unavailable until an operator
supplies a distinct scoped credential. Command details live in [temporary
runtime](temporary-runtime.md).

## Lifecycle

A service creates a session, creates the temporary instance, starts it, waits for
readiness, transfers participants, and stops it on success, timeout, disconnect,
empty state, or cancellation. After retention, the daemon deletes or archives
the world directory according to configuration.

## Atomic service rule

Point deduction, session creation, and instance creation must commit in one
daemon-side transaction. If process start later fails, the daemon refunds with
a deterministic refund correlation distinct from the spend correlation, marks
the session refunded, and audits the transition exactly once.

## Failure behavior

Startup, readiness, transfer, cancellation, and cleanup failures are explicit
states with operator diagnostics and player-safe messages. Cancellation stops the
real runtime before its durable state changes and uses the same idempotent refund
path when eligible. An unhealthy or fenced recovered identity is unverifiable,
so cancellation fails closed without durable stop, cancellation, or refund writes.
A failed or interrupted cleanup returns to a retryable state; only successful
world, instance-directory, and port cleanup is reported as done.

## Verification

Store helpers and planners need deterministic tests before daemon handlers.
Opt-in live smokes must use real PostgreSQL, Velocity, Folia, plugin install,
transfer, timeout, cleanup, and refund paths.

## Current status

The PostgreSQL tables, typed store helpers, pure state records, pure port and
world allocation planner, daemon local runtime create/start/stop/get/cleanup
commands, cleanup worker, Velocity registration hints, and daemon-validated
Velocity transfer intents exist. End Expedition and adventure catalog purchases
create sessions, charge points, start Folia instances, transfer players, and
refund startup failures through real daemon paths.

## Current boundary

The shipped boundary is local Folia temporary runtime with Velocity transfer
hints and opt-in live smokes. Additional adventure products must provide distinct
implemented gameplay and reuse the same transaction, transfer, return, cleanup,
and refund contracts before registration.
