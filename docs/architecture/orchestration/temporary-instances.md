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
record.

## Runtime contract

The daemon allocates unique ports, creates generated worlds, renders
Folia-compatible configuration without a root daemon-token path, installs
verified local-safe plugins, starts the process, and waits for readiness. Java
daemon credentials, Velocity registration, and player transfer are withdrawn
pending trusted identity/session attestation.

## Lifecycle

A daemon service can create a session, create and start an instance, wait for
readiness, and stop it on timeout, cancellation, or cleanup policy. It records
no player transfer. After retention, the daemon deletes or archives the world
according to configuration.

## Atomic service rule

Point deduction, session creation, and instance creation commit in one daemon
transaction. A later process-start failure refunds with a deterministic distinct
correlation, marks the session refunded, and audits once.

## Failure behavior

Startup, readiness, cancellation, and cleanup failures are explicit states.
Cancellation stops the real runtime before durable state changes. A failed or
interrupted cleanup returns to a retryable state; only successful world,
instance-directory, and port cleanup is reported as done.

## Verification

Store helpers, planners, and daemon handlers have deterministic tests. Java
containment inspection proves no registration or transfer adapter is packaged.
