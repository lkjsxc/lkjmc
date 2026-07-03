# Process runtime

## Purpose

This document defines the local process runtime contract.


## Status

implemented

## Rules

- Instances live under `/var/lib/lkjmc/instances/{id}`.
- Source instance config lives under `/etc/lkjmc/instances/{id}.json`.
- Logs live under `/var/log/lkjmc/instances/{id}`.
- Server jars come from `/opt/lkjmc/jars`.
- Process groups are used for reliable stop and kill.
- Graceful stop uses stdin or RCON when available, then signals, then kill.
- Deletion refuses active player sessions unless forced and audited.

## Current implementation

The daemon owns a local runtime for explicit launch commands stored in instance
config JSON. `instance.start` sets desired state to `running`, starts the
process in a new process group, writes `current.log`, records an observation,
and writes an audit event. `instance.stop` sends `TERM` to the process group,
waits with a bounded timeout, sends `KILL` if needed, and records absence.
`instance.restart`, `instance.list`, `instance.delete`, and `instance.logs` use
that same runtime state.

A periodic reconciler runs when a database URL is configured. It compares
desired state to tracked runtime state for explicit launch profiles and starts,
stops, or completes restarts. On daemon startup, live process groups from stored
healthy observations are recovered as detached runtime entries.

The runtime renders each instance directory before launch. It loads optional
JSON templates from `/etc/lkjmc/templates/{template}.json`, merges template and
instance properties, writes declared template files, and then writes platform
files. Paper, Folia, vanilla, and modded instances receive `eula.txt` and
`server.properties`. Velocity instances receive `velocity.toml`. Instance create reserves an
explicit server port or allocates the first free local port from the default
range and stores it in PostgreSQL-backed config. Launches run with the instance
directory as their working directory. Stop first attempts configured RCON
`stop`, then writes `stop` to process stdin when available, then escalates to
process-group signals after a bounded wait.

## Current boundaries

- Launch profiles are command arrays or verified jar asset IDs in JSON.
- Template rendering supports JSON file templates and built-in platform files,
  but does not hot-reload running processes.
- Recovered process handles can be stopped by process group, but stdout and
  stderr ownership remains with the original process.
- RCON stop requires a config `rcon` object with host, port, and password.
- Delete refuses a running process or active player sessions unless forced.
