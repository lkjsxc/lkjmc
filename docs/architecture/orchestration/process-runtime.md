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

Each local instance has an independent lifecycle guard and process entry.
Launch resolves the executable, records its device/inode identity and Linux
`/proc` start time, starts a new process group, and verifies those values before
reporting startup. Readiness and startup have bounded deadlines. A missing
`/proc` identity, changed executable, reused PID, or mismatched start time is
unhealthy and fenced, never adopted from a numeric PID or PGID alone.

Stop revalidates identity before writing `stop` or signalling. It waits to a
graceful deadline, sends `TERM`, then `KILL`, and reaps the child. Absence is
recorded only after the proved process identity and group are gone. Signal,
identity, wait, and deadline failures retain unknown or failed state for retry.
Shutdown closes lifecycle admission, stops every independently tracked child,
and joins cleanup before returning.

PostgreSQL stores each intent before launch/stop and releases its connection
before the process effect. The adapter returns an observation containing the
proved identity. A fresh transaction commits it only while operation and fence
ownership still match. Recovery observes a pending operation and the persisted
identity; unverifiable survivors are fenced and never signalled.

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

## Verification

Required runtime probes use real concurrent child process groups and disposable
PostgreSQL. They cover one-instance races, a hung unrelated instance, crash
windows, identity mismatch, stop escalation, idempotent reconciliation, and
zero-survivor shutdown. Missing PostgreSQL fails these named probes. The RCON
file test launches under `umask 000` and verifies the private-file mode.

## Current boundaries

- Launch profiles are command arrays or verified jar asset IDs in JSON.
- Template rendering supports JSON file templates and built-in platform files,
  but does not hot-reload running processes.
- A fenced unverifiable identity needs operator investigation; the daemon will
  not signal it.
- A live daemon Unix socket is owned by its listener: a second daemon refuses it
  rather than unlinking it. Only a stale socket file may be removed.
- RCON stop requires a config `rcon` object with host, port, and `passwordFile`.
  The password is never retained in instance JSON.
- Delete refuses a running process or active player sessions unless forced.
