# Backup and restore

## Purpose

This runbook defines real PostgreSQL backup and restore operations for lkjmc.

## Status

implemented

## Backup

Set `LKJMC_DATABASE_URL` to the operator database URL and write a custom-format
dump with restrictive permissions:

```sh
LKJMC_DATABASE_URL=postgres://... scripts/backup-postgres.sh backup.dump
```

The script runs `pg_dump --format=custom --no-owner` and never prints database
passwords or token files. Back up these filesystem paths separately with your
normal host backup tool: JSON config, daemon token files, forwarding secret,
asset and jar registries, templates, and Minecraft world data.

## Restore drill

Restore into an empty test database first:

```sh
LKJMC_DATABASE_URL=postgres://... scripts/restore-postgres.sh backup.dump
```

The script runs `pg_restore --clean --if-exists --no-owner`. It changes the
target database destructively; do not use a production URL for the drill. After
restore, run `lkjmc db status`, `lkjmc doctor`, and the relevant smoke checks
before pointing players at the deployment. A successful restore command proves
only that `pg_restore` completed, not that runtime processes, worlds, tokens, or
player routes were recovered; validate those separately.
