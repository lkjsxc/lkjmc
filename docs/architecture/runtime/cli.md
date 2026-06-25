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
- `lkjmc jar prune --yes`
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

Player restore is implemented as a daemon-backed immutable snapshot copy that
promotes a selected snapshot ID to the latest profile revision. Moderation CLI
commands call daemon-backed report and punishment APIs. `lkjmc verify` runs the
repository verification script in the current checkout and fails with that
script's status. Instance start supports explicit launch commands and verified
jar assets, with template-backed platform rendering for new instance directories.
