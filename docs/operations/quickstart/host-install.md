# Host deployment entrypoint

## Status

Existing-system immutable update is the maintained path. Clean host installation is not currently
supported.

## Existing deployment

Acquire and independently verify one exact release, retain its manifest SHA-256 out of band, make
the operator-owned host snapshot, and invoke that release's Rust authority:

```sh
sudo "$RELEASE/source/lkjmc-ops" deploy update \
  --operation-id "$OPERATION_UUID" \
  --release-root "$RELEASE" \
  --manifest-sha256 "$MANIFEST_SHA256" \
  --from-commit "$CURRENT_COMMIT" \
  --from-manifest-sha256 "$CURRENT_MANIFEST_SHA256" \
  --config /etc/lkjmc/lkjmc.json \
  --backup "/var/backups/lkjmc/pre-$NEW_COMMIT.dump" \
  --rollback-snapshot "$OPERATOR_SNAPSHOT_LABEL"
```

See [immutable update and recovery](../install.md) for preflight, exact no-op, fence, rollback, and
interruption behavior. The updater does not provision secrets, assets, PostgreSQL, or networking.
The explicit root-owned EULA policy is created separately for an operator-owned host.

## Clean host

No clean installer is shipped. Until a disposable unprivileged-system-container drill independently
covers service identity, private roots, PostgreSQL, exact assets, dynamic credentials, host EULA
policy, systemd restart, backup, and isolated restore for an exact release, perform no unattended
clean installation and make no playable or production claim.
