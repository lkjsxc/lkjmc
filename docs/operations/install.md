# Install

## Purpose

Define truthful, rerunnable installation and rollback ownership.

## Status

implemented

## Host provisioner

`scripts/install.sh` provisions an Ubuntu-like system installation from a
checkout. It requires root, creates the system account and PostgreSQL database,
writes private generated secrets without printing them, applies migrations,
installs release artifacts, and starts a real daemon through systemd or the
owned fallback supervisor. Success requires the service-user socket status
query; service-manager configuration or process creation alone is never
success.

The provisioner resolves numeric UID/GID through system account databases,
refuses inaccessible or ambiguously owned source, and never changes checkout
ownership. Secrets and database environment are service-owned mode `0600`.
Writable runtime, asset, jar, data, and log roots are service-owned; installed
binaries and configuration are not writable by the daemon account.

## Artifact installer scopes

`scripts/install-artifacts.sh` installs a prebuilt manifest with one scope:

- `system`: root only, explicit service UID/GID, private system roots;
- `user`: non-root only, paths below the selected user root;
- `rootless`: non-root only, user-owned paths, no setuid/setgid files, no system
  service-manager claim, and an externally supplied PostgreSQL URL.

All scopes verify the artifact manifest before mutation, stage on the same
filesystem, fsync regular files, and rename into place. Existing installed bytes
move to one private rollback directory. Any copy, checksum, ownership, mode, or
post-install status failure restores the prior tree and removes staging. A rerun
with identical input is a no-op except for a fresh verified status check;
generated secrets are neither replaced nor printed.

## Verification

`LKJMC_INSTALLER_SMOKE=1 ./scripts/check-installer.sh` uses an isolated host
container and runs the system provisioner twice. The operations checker also
runs user and rootless artifact scopes twice and injects copy/checksum/status
failure. It checks no old daemon survives, private modes and owners do not
drift, source ownership is unchanged, secrets are unchanged and absent from
logs, rollback restores the prior checksum, and no staging path survives.

## Playable and external boundaries

`--playable --accept-minecraft-eula` is explicit consent before writing the
Minecraft EULA or starting an adapter. Rootless service supervision, production
systemd policy, public listeners, PostgreSQL provisioning by a cloud provider,
firewall/DNS changes, and platform package installation are external
prerequisites. Missing prerequisites are skips or failures, never fabricated
service success.
