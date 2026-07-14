# Bootstrap effects

## Purpose

This contract defines the effect names and ordering used by daemon
bootstrap adapters.


## Status

implemented

## Effect families

- `root.ensure`: create product roots with safe permissions.
- `database.migrate`: apply schema migrations.
- `secret.generate`: create HTTP token or forwarding secret files.
- `asset.server.sync`: download and verify PaperMC server jars.
- `asset.plugin.sync`: download or register verified plugin jars.
- `instance.reconcile`: create or update managed instance metadata.
- `template.render`: write Velocity, Paper, and plugin configuration files.
- `plugin.install`: copy immutable assets into instance plugin directories.
- `instance.start`: start Java processes through the daemon runtime.
- `probe.wait`: wait for process, port, log, and status readiness.

## Selected imperative ordering

The sole product path is `bootstrap.plan`/`bootstrap.apply` over the parsed
`network` JSON object. Ordered effects are lock, preflight, durable desired
record, roots, secrets, verified assets, instance metadata, atomic restrictive
render, backend start/readiness, proxy start/readiness, observation, and terminal
history. Installer and Compose invoke this daemon path; neither launches Java
or compiles manifests independently.

Each file is rendered to a run-owned sibling, flushed, permissioned to owner
read/write only, and atomically renamed. Apply uses one bounded lock deadline
and one overall deadline. It never holds a pooled database connection while
waiting on filesystem, process, listener, readiness, or Kubernetes work.

## Truthfulness

An effect returns success only after the filesystem, database, process, or probe
operation completed and any required validation passed. Restart propagates a stop
failure, port reservation conflicts fail, and readiness requires the current
instance log identity at `/var/log/lkjmc/{id}/current.log` plus its configured
listener. Before `probe.wait`, apply records a running probe step, then releases its pooled
PostgreSQL connection for the wait. Its separate PostgreSQL advisory-lock
session remains held from admission through terminal probe bookkeeping, so a
second process cannot apply while readiness waits. Apply reconnects to record
the terminal probe result. A post-wait bookkeeping failure is an apply error
after a successful probe; a failed probe remains its own reported error. Local
Unix command requests allow the full bounded readiness window; TCP requests keep
the normal short transport deadline.

An unset database URL makes status report the database-unavailable skip. An
explicitly empty database URL is a configuration error, never an unavailable
skip. Daemon apply code must match every `BootstrapEffect` variant exhaustively;
adding a variant without a real adapter is a compile-time review blocker, not a
catch-all success path.

## Ledger boundary

`database.migrate` may run before bootstrap ledger tables exist. If migration
fails before those tables are created, the daemon returns the real migration
error without fabricating a run record. After migrations are available, one
PostgreSQL advisory lock admits a single apply; stale running runs are failed
before the next apply and every remaining effect is recorded in `bootstrap_steps`.
