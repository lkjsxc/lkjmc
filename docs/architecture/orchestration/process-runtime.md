# Process runtime

## Purpose

This document defines the local process runtime target contract.

## Rules

- Instances live under `/var/lib/lkjmc/instances/{id}`.
- Source instance config lives under `/etc/lkjmc/instances/{id}.json`.
- Logs live under `/var/log/lkjmc/instances/{id}`.
- Server jars come from `/opt/lkjmc/jars`.
- Process groups are used for reliable stop and kill.
- Graceful stop uses stdin or RCON when available, then signals, then kill.
- Deletion refuses active player sessions unless forced and audited.

## Current implementation

The daemon owns an in-memory local runtime for explicit launch commands stored
in instance config JSON. `instance.start` sets desired state to `running`, starts
the process in a new process group, writes `current.log`, records an observation,
and writes an audit event. `instance.stop` sends `TERM` to the process group,
waits with a bounded timeout, sends `KILL` if needed, and records absence.
`instance.restart`, `instance.list`, `instance.delete`, and `instance.logs` use
that same runtime state.

## Current boundaries

- Reconciliation is command-driven; no periodic desired-state loop runs yet.
- Runtime state is in memory and is not recovered after daemon restart.
- Launch profiles are command arrays in JSON; jar and template rendering are not
  connected yet.
- Graceful stdin and RCON stop paths are not implemented yet.
- Delete refuses a running process unless forced, but active player session
  guards are not implemented yet.
