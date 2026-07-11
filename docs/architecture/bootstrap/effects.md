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

## Default ordering

The playable order is roots, migrations, secrets, local plugin assets, server
assets, third-party assets, hub reconcile, proxy reconcile, backend render,
proxy render, hub start, hub readiness, proxy start, proxy readiness, final
status refresh.

## Truthfulness

An effect returns success only after the filesystem, database, process, or probe
operation completed and any required validation passed. Restart propagates a stop
failure, port reservation conflicts fail, and readiness requires the current
instance log identity at its configured path plus its configured listener. Daemon
apply code must match every `BootstrapEffect` variant exhaustively; adding a
variant without a real adapter is a compile-time review blocker, not a catch-all
success path.

## Ledger boundary

`database.migrate` may run before bootstrap ledger tables exist. If migration
fails before those tables are created, the daemon returns the real migration
error without fabricating a run record. After migrations are available, one
PostgreSQL advisory lock admits a single apply; stale running runs are failed
before the next apply and every remaining effect is recorded in `bootstrap_steps`.
