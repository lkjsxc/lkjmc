# Plugin provisioning

## Inventory-derived installation

`lkjmc-ops` enumerates targets from each configured instance's typed integration:

- `velocity` receives `lkjmc-velocity.jar`;
- `paper-compatible` receives `lkjmc-paper.jar`;
- `none` receives no lkjmc Java plugin.

Every destination is below `dataRoot/instances/<instance-id>/plugins` and must byte-match the
anchored release manifest. The updater installs jars only while the systemd service is stopped and
verifies their final type, owner, mode, size, and SHA-256. Backend names and counts do not select
artifacts.

## Scoped heartbeat credentials

Each integrated instance has one derived path:
`dataRoot/private/plugin-credentials/<instance-id>.secret`. Generated launch state supplies only
bounded nonsecret values such as instance ID/kind, server port, loopback heartbeat endpoint,
credential-file path, and—for Velocity—the deterministic backend-ID list. The child environment is
cleared before spawn, so daemon/PostgreSQL credentials are not inherited.

An authenticated operator creates a credential with principal kind `instance`, a surface matching
the instance's persisted kind, exactly `lkjmc.instance.heartbeat`, and the canonical
instance-bound `.secret` or `.next.secret` path. The daemon queries the managed fleet; arbitrary
IDs cannot claim Paper or Velocity authority. The value is written once with private metadata and is
never returned or logged.

Rotation creates `<id>.next.secret`, verifies the returned fingerprint and metadata, atomically
replaces the canonical path, observes heartbeat recovery, then revokes the old credential ID.
Unknown commit status retains the candidate for reconciliation. Distinct credentials narrow API
authority and audit/revocation scope; they are not process isolation between malicious components
sharing one Unix account.
