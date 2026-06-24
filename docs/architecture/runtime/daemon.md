# Daemon

## Purpose

This document defines implemented and target daemon responsibilities.

## Implemented API

`lkjmc-daemon` starts a Unix socket JSON-RPC server. It accepts one newline
terminated JSON envelope per connection and returns one JSON response.

Implemented commands:

- `doctor`
- `status`
- `audit.tail` when the daemon has a database URL
- `instance.list`
- `instance.create`
- `instance.start`
- `instance.stop`
- `instance.restart`
- `instance.delete`
- `instance.logs`

The daemon also has a loopback HTTP listener for plugin clients. HTTP requests
must include a bearer token when the listener is enabled with a token.

At startup the daemon may load `/etc/lkjmc/lkjmc.json` or a path provided by
`--config`. Command-line flags remain available for tests and local overrides.
The daemon reads the PostgreSQL secret file referenced by JSON config and never
prints the derived connection string.

## Current process runtime

Instance commands use PostgreSQL and a local process runtime. Instances with an
explicit launch command can be started as process groups, stopped, restarted,
observed, deleted with running-process and active-session guardrails, and tailed
from bounded log files. The daemon renders minimal instance files before launch
and attempts configured RCON plus stdin `stop` before signal escalation. A
periodic reconciler keeps explicit launch-command instances aligned, and daemon
startup recovers live process groups from stored healthy observations.

## Current boundaries

The daemon does not render full template registry content yet. Jar download
sync and filesystem config loading are implemented slices.
