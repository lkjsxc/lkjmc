# Backup and restore

## Purpose

Define transaction-consistent PostgreSQL backup and automated fresh restore.

## Status

implemented

## Backup contract

Set `LKJMC_DATABASE_URL` and write a custom-format dump:

```sh
LKJMC_DATABASE_URL=postgres://... scripts/backup-postgres.sh backup.dump
```

The script opens one repeatable-read transaction, exports its PostgreSQL
snapshot, records the WAL LSN and `schema_migrations` identity from that same
snapshot, and passes the snapshot to `pg_dump --format=custom --no-owner`. The
migration rows are an ordered `jsonb_agg`, read through unaligned tuples-only
`psql`, parsed with exact field and ordering checks, and serialized as compact
single-line canonical JSON before hashing. It writes private `backup.dump`,
`backup.dump.metadata.json`, and `backup.dump.sha256` files only after dump and
manifest checks complete. Metadata records the server version, source commit,
LSN, schema, migration, dump, and manifest SHA-256 values. It contains no URL,
password, or token. Interrupted or partial work is removed.

`pg_dump`, `pg_restore`, and `psql` major versions must agree with the recorded
server major. Checksum, metadata schema, source migration availability, and dump
manifest are fail-closed restore inputs. PostgreSQL is the only product store.
Back up JSON configuration, token files, asset and jar registries, templates,
and Minecraft worlds separately using private operator storage.

## Fresh restore and boot drill

`scripts/restore-postgres.sh backup.dump` refuses a database containing user
relations. It validates checksums and versions before `pg_restore --no-owner`,
applies committed migrations, then compares the restored migration marker.
The operations lab creates a unique fresh database, restores, migrates, starts
the actual daemon with a private generated configuration and socket, and runs
readiness, `db status`, `doctor`, and a direct PostgreSQL query. It repeats the
complete backup/restore/boot path twice and repeats owned-resource cleanup.

The drill injects corrupted dump, unsupported metadata version, and partial
metadata failures. Every case must fail without reporting readiness and cleanup
must drop the fresh database, stop the daemon, remove its socket, and remove
partial output.

## Rollback boundary

A restore never mutates the source database. Production rollback means stop
writers, retain the failed database, create a fresh target from the last
verified backup, validate it, then atomically switch the private database
configuration under a change record. Never overwrite the only copy or call a
successful `pg_restore` alone a service recovery.
