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
config JSON. `instance.start` starts in a new process group, truncates its run log, and
requires a healthy post-start observation. An absent or unhealthy observation
fails the command and does not set desired state to `running`; at most two
post-start attempts use a bounded 100 ms retry delay. `instance.stop` sends
`TERM` to the process group, waits with a bounded timeout, sends `KILL` if
needed, and records
absence only after it confirms that the group is gone. A signal or wait failure
keeps the entry tracked so a retry cannot claim the possibly live group absent.
`instance.restart`, `instance.list`, `instance.delete`, and `instance.logs` use
that same runtime state.

A periodic reconciler runs when a database URL is configured. It compares
desired state to tracked runtime state for explicit launch profiles and starts,
stops, or completes restarts. A stored PID is never adopted after daemon restart:
the daemon cannot prove that a reused PID and process group belong to the prior
launch, so it records an unhealthy fenced observation and refuses to signal or
replace that identity automatically.

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

Local tests create process groups to prove fencing and stop escalation.
PostgreSQL-backed create, retry, recovery, and cancellation tests run only when
`LKJMC_STORE_TEST_DATABASE_URL` is a disposable database; without it they skip
rather than proving durable writes. The RCON file test launches under `umask 000`
and verifies the private-file mode.

## Current boundaries

- Launch profiles are command arrays or verified jar asset IDs in JSON.
- Template rendering supports JSON file templates and built-in platform files,
  but does not hot-reload running processes.
- A fenced recovered PID needs operator investigation; the daemon will not
  signal an unverifiable process group.
- A live daemon Unix socket is owned by its listener: a second daemon refuses it
  rather than unlinking it. Only a stale socket file may be removed.
- RCON stop requires a config `rcon` object with host, port, and `passwordFile`.
  The password is never retained in instance JSON.
- Delete refuses a running process or active player sessions unless forced.
