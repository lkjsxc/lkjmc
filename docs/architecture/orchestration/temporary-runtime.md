# Temporary runtime

## Purpose

This contract defines daemon commands for temporary instance runtime
lifecycle.


## Status

implemented

## Commands

The daemon owns temporary create, start, stop, cleanup, and status commands.
Create allocates a hidden Folia instance, unique port, generated world path,
strict lifetime, retention policy, and verified jar reference. Start installs
the required `lkjmc` Paper plugin, starts the local process, waits for
readiness, and records failure truthfully. Stop records a stopped lifecycle
state only after process-group absence is confirmed. Cleanup removes or archives
generated world and instance-directory data only after retention or an explicit
force flag, then releases its port.

## Data and effects

Creation writes normal `instances`, `instance_ports`, and `temporary_instances`
rows in one daemon-side operation. Filesystem world creation is rolled back when
the database write fails. Process and filesystem effects are never reported as
successful until completed.

## Transfer contract

A transfer intent command may record a target for a future attested consumer,
but no Velocity bridge consumes it. Failed validation returns a daemon error and
must not attempt a proxy transfer.

## Failure behavior

Port exhaustion, existing world directories, missing Folia jars, missing plugin
assets, missing forwarding secrets, startup failure, readiness timeout, transfer
to non-ready instances, running cleanup targets, and failed deletion all return
daemon errors and audit failed transitions when possible.

## Current status

The daemon commands are implemented for the local runtime: create, start, stop,
get, explicit cleanup, and transfer intent. A daemon worker stops expired
instances and deletes or archives retained worlds. The runtime uses PostgreSQL,
generated world directories, verified Folia jars, local-safe `lkjmc` Paper plugin
install, identity-bound readiness probes, retention checks, and audit events.
Java daemon adapters, Velocity registration hints, and transfer handoff are
withdrawn pending trusted identity/session attestation. Readiness waits do not
retain a database pool connection.

## Current boundary

Velocity dynamic registration and Java adventure menu/transfer paths are
withdrawn pending trusted identity/session attestation. Daemon temporary runtime
records do not make a Java proxy registration or player transfer occur.
