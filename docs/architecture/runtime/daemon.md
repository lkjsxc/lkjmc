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

## Current process runtime

Instance commands use PostgreSQL and an in-memory local runtime. Instances with
an explicit launch command can be started as process groups, stopped, restarted,
observed, deleted with guardrails, and tailed from bounded log files.

## Current boundaries

The daemon does not run a periodic desired-state reconciler, recover process
state after restart, download jars, render templates, load config files from
disk, or perform active player deletion checks yet.
