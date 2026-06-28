# Temporary runtime

## Purpose

This target contract defines daemon commands for temporary instance runtime
lifecycle.

## Commands

The daemon owns temporary create, start, stop, cleanup, and status commands.
Create allocates a hidden Folia instance, unique port, generated world path,
strict lifetime, retention policy, and verified jar reference. Start installs
the required `lkjmc` Paper plugin, starts the local process, waits for
readiness, and records failure truthfully. Stop stops the process and records a
stopped lifecycle state. Cleanup removes or archives generated world data only
after retention or an explicit force flag.

## Data and effects

Creation writes normal `instances`, `instance_ports`, and `temporary_instances`
rows in one daemon-side operation. Filesystem world creation is rolled back when
the database write fails. Process and filesystem effects are never reported as
successful until completed.

## Failure behavior

Port exhaustion, existing world directories, missing Folia jars, missing plugin
assets, missing forwarding secrets, startup failure, readiness timeout, running
cleanup targets, and failed deletion all return daemon errors and audit failed
transitions when possible.

## Current status

The daemon commands are implemented for the local runtime: create, start, stop,
get, and explicit cleanup. They use PostgreSQL, generated world directories,
verified Folia jars, verified `lkjmc` Paper plugin install, readiness probes,
retention checks, and audit events.

## Current boundary

Velocity dynamic registration, player transfer, End Expedition purchase, refund,
and cleanup worker scheduling remain separate blockers. Live player purchase
surfaces stay disabled until those paths are implemented.
