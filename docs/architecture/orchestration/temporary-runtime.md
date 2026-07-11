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

A transfer intent command validates that the temporary instance is ready and
registered, records the player target intent with a short expiry, and returns the
target server id. Velocity consumes that daemon success before using its existing
profile-safe transfer bridge. Failed validation returns a daemon error and must
not attempt a proxy transfer.

## Failure behavior

Port exhaustion, existing world directories, missing Folia jars, missing plugin
assets, missing forwarding secrets, startup failure, readiness timeout, transfer
to non-ready instances, running cleanup targets, and failed deletion all return
daemon errors and audit failed transitions when possible.

## Current status

The daemon commands are implemented for the local runtime: create, start, stop,
get, explicit cleanup, and transfer intent. A daemon worker stops expired
instances and deletes or archives retained worlds. The runtime uses PostgreSQL,
generated world directories, verified Folia jars, verified `lkjmc` Paper plugin
install, identity-bound readiness probes, retention checks, audit events,
Velocity registration hints, and profile-safe Velocity transfer handoff.
Readiness waits do not retain a database pool connection.

## Current boundary

Velocity dynamic registration uses daemon `instance.list` registration hints.
End Expedition has daemon purchase, solo, party, return, startup refund, and
menu/catalog paths. It is the only shipped catalog adventure; generic generated
world labels are withdrawn until they have distinct implemented gameplay.
