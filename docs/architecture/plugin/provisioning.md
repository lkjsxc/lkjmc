# Plugin provisioning

## Current artifacts

The initial network installs only two project-built shaded plugin artifacts:

```text
/var/lib/lkjmc/instances/proxy/plugins/lkjmc-velocity.jar
/var/lib/lkjmc/instances/hub/plugins/lkjmc-paper.jar
/var/lib/lkjmc/instances/survival/plugins/lkjmc-paper.jar
```

Each installed file must byte-match the exact release manifest. Replacement occurs only while the three-instance service is stopped, followed by systemd bootstrap reconciliation and platform startup evidence. Dormant third-party plugin catalogs are not a supported installation surface.

## Heartbeat credentials

Each process receives only these non-secret/scoped environment values from its generated instance config:

```text
LKJMC_INSTANCE_ID
LKJMC_INSTANCE_KIND
LKJMC_SERVER_IMPLEMENTATION
LKJMC_SERVER_PORT
LKJMC_HEARTBEAT_ENDPOINT=http://127.0.0.1:8765/plugin/v1/heartbeat
LKJMC_HEARTBEAT_CREDENTIAL_FILE=/var/lib/lkjmc/private/plugin-credentials/<id>.secret
```

The runtime clears the daemon's inherited environment before spawning Java. In particular, PostgreSQL and daemon bootstrap credentials cannot reach a plugin process through the parent environment.

An authenticated local operator creates three distinct credentials (`proxy`, `hub`, and `survival`) through `lkjmc security token create`. Plugin credentials use principal kind `instance`, a surface matching the platform, exactly the `lkjmc.instance.heartbeat` scope, a maximum one-year expiry, and an ID-bound canonical or `.next.secret` output path. The daemon makes the immediate credential directory mode `0700`, writes the value once with mode `0600`, and never returns or logs it.

Rotation is ordered: create a new credential at `<id>.next.secret`; verify its fingerprint response and private metadata; atomically rename it over `<id>.secret`; observe heartbeat recover with the new token; then revoke the old credential ID. Revocation intentionally does not remove a file because the canonical path may already contain the replacement. If creation reports unknown commit status, preserve the `.next.secret` file, inspect the credential/audit rows locally, and reconcile before any rename or retry. A failed rotation leaves the existing canonical token in place.

All components inside the service container remain practically trusted and currently share the `lkjmc` Unix account. Distinct credentials therefore limit daemon API authority and make revocation/audit explicit; they are not claimed as process isolation from another malicious same-UID component.
