# CLI

## Purpose

This document defines the implemented and target SSH-friendly operator surface.

## Implemented commands

- `lkjmc doctor`
- `lkjmc status`
- `lkjmc config check --path PATH`
- `lkjmc db migrate`
- `lkjmc db status`
- `lkjmc audit tail --lines N`
- `lkjmc instance list`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE --command CMD`
- `lkjmc instance start ID`
- `lkjmc instance stop ID`
- `lkjmc instance restart ID`
- `lkjmc instance delete ID --yes [--force]`
- `lkjmc instance logs ID --lines N`

`doctor`, `status`, `audit tail`, and instance commands use the daemon Unix
socket. Database migration and status use `LKJMC_DATABASE_URL` directly.
`--json` emits compact machine-readable JSON for implemented commands.

## Current boundaries

Jar, player restore, and jar-backed instance launch commands are not implemented
or registered yet. Instance start currently requires a JSON launch command; it
does not resolve jar assets or render templates.
