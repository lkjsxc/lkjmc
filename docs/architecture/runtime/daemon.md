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

The daemon also has a loopback HTTP listener for plugin clients. HTTP requests
must include a bearer token when the listener is enabled with a token.

## Current boundaries

The daemon does not reconcile instances, supervise processes, download jars,
render templates, or load config files from disk yet.
