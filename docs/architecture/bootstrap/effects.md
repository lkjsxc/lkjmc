# Bootstrap effects

## Purpose

This target contract defines the effect names and ordering used by daemon
bootstrap adapters.

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
operation completed and any required validation passed.
