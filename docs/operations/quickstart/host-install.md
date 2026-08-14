# Host deployment entrypoint

## Status

Existing-system immutable update is implemented. Clean host installation is not
yet supported.

## Existing deployment update

Build and privately transfer one clean release, preserve the manifest SHA-256
out of band, create an Incus snapshot on the host, then invoke the deployer from
the release itself:

```sh
sudo "$RELEASE/source/lkjmc-deploy-release" update \
  --release-root "$RELEASE" \
  --manifest-sha256 "$MANIFEST_SHA256" \
  --from-commit "$CURRENT_COMMIT" \
  --backup "/var/backups/lkjmc/pre-$NEW_COMMIT/lkjmc.dump" \
  --rollback-snapshot "$INCUS_SNAPSHOT"
```

See [immutable release update](../install.md) for all preconditions, rollback
behavior, and exact success evidence. A second invocation for the same release
is a verified no-op and does not restart the JVMs.

## Clean host

`scripts/install.sh` is deliberately withdrawn. It previously built ambient
checkout output, wrote an obsolete topology, and had no complete migration
rollback boundary. It now exits before mutation.

Until a disposable unprivileged-LXC clean-install drill covers PostgreSQL,
service identity, private roots and secrets, exact Velocity/Folia assets, three
scoped heartbeat credentials, an existing EULA record, systemd restart, backup,
and restore, perform no unattended clean installation and make no playable
claim.

The updater does not create secrets or acceptance records and never prints
credential values. It preserves existing database, daemon HTTP, forwarding, and
instance credentials during an ordinary update.
