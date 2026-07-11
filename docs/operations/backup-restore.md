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

1. Set `SOURCE_CONFIG` to the source JSON config, then create the lab copy:

   ```sh
   SOURCE_CONFIG=/path/to/lkjmc.json
   LAB_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/lkjmc-restore.XXXXXX")
   LAB_CONFIG="$LAB_ROOT/lkjmc.json"
   cp "$SOURCE_CONFIG" "$LAB_CONFIG"
   ```

2. Set `LAB_DATABASE_URL` to the isolated database URL, then restore with
   `LKJMC_DATABASE_URL="$LAB_DATABASE_URL" scripts/restore-postgres.sh backup.dump`.
3. Edit only the lab copy. Its `database.host`, `database.port`,
   `database.database`, `database.user`, and `database.secretFile` must resolve
   to that lab database; `--config` takes these values instead of the environment
   URL. Resolve token-file, runtime roots, and namespace to lab-only resources.
4. Set `lab_socket="$LAB_ROOT/daemon.sock"` and validate the exact copy with
   `lkjmc config check --path "$LAB_CONFIG"`.
5. Start `lkjmc-daemon --config "$LAB_CONFIG" --socket "$lab_socket" --http none`
   in the background, wait until `[ -S "$lab_socket" ]`, then run
   `lkjmc --socket "$lab_socket" db status` and `lkjmc --socket "$lab_socket" doctor`.
6. Retain redacted output, stop the daemon, and confirm the lab socket is gone.

This proves that the restored database can boot the daemon under the resolved,
isolated configuration. It does not prove runtime processes, worlds, tokens,
external routes, capacity, or player recovery. Validate those separately; see
[clean-room lab](clean-room-lab.md) for the evidence boundary.
