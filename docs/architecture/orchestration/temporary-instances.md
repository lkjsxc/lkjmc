# Temporary instances

## Purpose

This target contract defines daemon-managed short-lived Minecraft backends.

## Data contract

Temporary instances are normal instance records plus durable temporary metadata:
owner command, visibility, generated world path, allocated ports, maximum
lifetime, autosuspend policy, retention policy, cleanup state, and audit
correlation. PostgreSQL is the source of truth; plugin memory is never the only
record of a temporary backend.

## Runtime contract

The daemon allocates unique ports, creates a generated world directory, renders
Folia-compatible config, installs verified required plugins, starts the process,
waits for readiness, registers it through Velocity, and hides it from normal
server listings unless a product explicitly exposes it. Command details live in
[temporary runtime](temporary-runtime.md).

## Lifecycle

A service creates a session, creates the temporary instance, starts it, waits for
readiness, transfers participants, and stops it on success, timeout, disconnect,
empty state, or cancellation. After retention, the daemon deletes or archives
the world directory according to configuration.

## Atomic service rule

Point deduction, session creation, and instance creation must commit in one
daemon-side transaction. If process start later fails, the daemon refunds
through the points ledger, marks the session failed, and audits the transition.

## Failure behavior

Startup, readiness, transfer, and cleanup failures are explicit states with
operator diagnostics and player-safe messages. A failed cleanup is retried by the
daemon and never reported as successful deletion.

## Verification

Store helpers and planners need deterministic tests before daemon handlers.
Opt-in live smokes must use real PostgreSQL, Velocity, Folia, plugin install,
transfer, timeout, cleanup, and refund paths.

## Current status

The PostgreSQL tables, typed store helpers, pure state records, pure port and
world allocation planner, daemon local runtime create/start/stop/get/cleanup
commands, and Velocity registration hints exist. Player transfer, cleanup
worker, and purchase commands are not implemented yet.

## Current boundary

No temporary adventure daemon commands or live purchase menu actions may be
registered until creation, transfer, stop, and cleanup work end to end.
