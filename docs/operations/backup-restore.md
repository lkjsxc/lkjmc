# PostgreSQL backup and restore

## Backup

`lkjmc-ops` is the packaged authority. It reads the canonical configuration and invokes fixed,
validated PostgreSQL tools directly with a bounded environment; it never shell-parses or logs a
database URL or password.

```sh
sudo /opt/lkjmc/releases/current/bin/lkjmc-ops database backup \
  --config /etc/lkjmc/lkjmc.json \
  --output /var/backups/lkjmc/manual.dump \
  --source-commit "$CURRENT_COMMIT"
```

The output path must not already exist. The operation produces private custom-format dump bytes,
metadata, and checksum closure. Metadata binds server version, source release, schema identity,
ordered migration identity, creation time bounds, dump SHA-256, and manifest SHA-256. Independent
`pg_restore --list` inspection is required before the receipt says `backup-verified`. The packaged
authority invokes the PostgreSQL 14 client binaries through fixed, root-owned paths;
the supported host must provide that exact client major. `pg_dump` or checksum alone is
insufficient.

Changed update records the planned backup destination before invoking PostgreSQL. If interruption
leaves no accepted closure, recovery removes only an exact same-owner, mode-`0700` staging directory
whose name binds the planned destination and a UUID. A verified final closure is retained and bound
into the journal even when the pre-fence operation is later classified `abandoned`.

Reverify a retained closure without mutation:

```sh
sudo /opt/lkjmc/releases/current/bin/lkjmc-ops database backup-verify \
  --config /etc/lkjmc/lkjmc.json \
  --backup /var/backups/lkjmc/manual.dump \
  --source-commit "$CURRENT_COMMIT" \
  --max-age-seconds 3600
```

Worlds, typed host configuration, credentials, and immutable Minecraft assets are separate backup
boundaries and are not placed in the PostgreSQL dump.

## Isolated restore verification

Restore acceptance uses a new empty database, never the live source:

```sh
sudo /opt/lkjmc/releases/current/bin/lkjmc-ops database restore-verify \
  --config /etc/lkjmc/lkjmc.json \
  --backup /var/backups/lkjmc/manual.dump \
  --source-commit "$CURRENT_COMMIT" \
  --target-database lkjmc_restore_probe
```

The target name is strictly validated and must contain no user relations. The command verifies the
backup closure, restores without owner changes, applies the exact committed migration sequence, and
compares schema and migration identity before reporting `restore-verified`. The isolated target is
operator-owned cleanup state; the failed/live database is retained until a replacement is accepted.

## Recovery boundary

An update failure after a changed or unreadable migration ledger remains fenced. Recovery requires
the journal-named backup and compatible release together; repointing only the binary is forbidden.
Replacing a live database, changing private connection configuration, and accepting the restored
service are explicit operator actions outside `database restore-verify`.
