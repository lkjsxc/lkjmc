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

All scopes verify the independently derived manifest closure before mutation,
stage on the same filesystem, fsync regular files and directories, and rename
into place. Existing installed bytes move to one private rollback directory.
Any copy, checksum, ownership, mode, version, or post-publish validation failure
restores the prior tree and removes staging.

An identical rerun compares every verified source hash with installed hash,
mode, numeric ownership, exact path set, and commit version. It performs a fresh
filesystem validation but does not replace files, touch timestamps, restart a
service, or invent service status; inode and mtime remain unchanged. A changed
release is atomically published and validated before rollback removal. Numeric
system UID/GID values need not have account-database names. Generated secrets
are neither replaced nor printed.

## Verification

`LKJMC_INSTALLER_SMOKE=1 ./scripts/check-installer.sh` uses an isolated host
container and runs the system provisioner twice. The operations checker also
runs system scope with an unnamed numeric GID plus user and rootless scopes.
Container drills execute non-root scopes as the numeric owner of the private
evidence mount rather than assuming a fixed image account can traverse it. They
inject copy, changed-release validation, and status-validation failures.
It checks identical rerun inode and mtime stability, private modes and owners,
source ownership, atomic changed updates, rollback to the exact prior tree, and
absence of staging paths. Artifact installation makes no daemon status claim.

## Playable and external boundaries

`--playable --accept-minecraft-eula` is explicit consent before writing the
Minecraft EULA or starting an adapter. Rootless service supervision, production
systemd policy, public listeners, PostgreSQL provisioning by a cloud provider,
firewall/DNS changes, and platform package installation are external
prerequisites. Missing prerequisites are skips or failures, never fabricated
service success.
