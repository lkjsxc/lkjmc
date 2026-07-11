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
target database destructively; do not use a production URL for the drill.

## Validation path

1. Create an empty isolated database and use its URL only for the restore.
2. Copy the JSON daemon config and resolve its database URL, token-file path,
   socket, runtime roots, and any namespace to lab-only resources.
3. Validate that exact copy with `lkjmc config check --path restore-lab.json`.
4. Start `lkjmc-daemon --config restore-lab.json --socket LAB_SOCKET --http none`
   with that resolved copy and lab socket.
5. Run `lkjmc --socket LAB_SOCKET db status` and `lkjmc --socket LAB_SOCKET
   doctor`; retain redacted output and stop the daemon afterward.

This proves that the restored database can boot the daemon under the resolved,
isolated configuration. It does not prove runtime processes, worlds, tokens,
external routes, capacity, or player recovery. Validate those separately; see
[clean-room lab](clean-room-lab.md) for the evidence boundary.
