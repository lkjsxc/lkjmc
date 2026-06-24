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
- `lkjmc jar list`
- `lkjmc jar import --kind KIND --name NAME --path PATH`
- `lkjmc jar sync --project PROJECT --channel stable [--version VERSION]`
- `lkjmc jar inspect QUERY`
- `lkjmc instance list`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE --command CMD`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE --jar-asset UUID`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE --server-port PORT`
- `lkjmc instance start ID`
- `lkjmc instance stop ID`
- `lkjmc instance restart ID`
- `lkjmc instance delete ID --yes [--force]`
- `lkjmc instance logs ID --lines N`

`doctor`, `status`, `audit tail`, and instance commands use the daemon Unix
socket. Database migration and status use `LKJMC_DATABASE_URL` directly.
`--json` emits compact machine-readable JSON for implemented commands.

## Current boundaries

Player restore and jar prune commands are not implemented or registered yet.
Instance start supports explicit launch commands and verified jar assets, but it
uses only minimal platform rendering until the full template registry exists.
