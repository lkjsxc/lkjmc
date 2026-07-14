# Install

## Purpose

This document defines current installer behavior and the playable installer
target.


## Status

implemented

## Target hosts

The host installer targets Ubuntu-like LXC and WSL2 systems where PostgreSQL,
Java 21, Rust, and Gradle can run.

## Current responsibilities

`scripts/install.sh` must be run as root from a checkout. It installs apt
packages, installs a current Rust toolchain when the distro Cargo is too old,
starts PostgreSQL, creates the `lkjmc` user and product roots, generates service
secrets without printing them, creates or updates the PostgreSQL role and
database, writes JSON config, builds and installs Rust binaries, applies
migrations, starts the daemon from that config with the checkout as working
directory through systemd when available, falls back to a WSL-style supervisor
command, waits for its socket, and reads `lkjmc status`. The general `doctor`
command remains denied-unproved and is not used to fabricate installer success.

The installer is idempotent for directories, writable jar and asset roots,
user creation, database creation, secret reuse and ownership, config writing,
migration application, and service restart. It resolves service ownership from
numeric UID/GID values and the system account databases; it never treats the
`stat` display value `UNKNOWN` as a group name. An unnamed checkout GID is not
added as a supplemental group. The installer proceeds only when the service
user can read and traverse that checkout, otherwise it fails with a remediation
before changing product ownership. Product ownership changes use the resolved
numeric service UID/GID.

Daemon CLI checks run as the service user rather than root so Unix-socket peer
authorization remains valid. Plugin jar installation remains limited to
artifacts produced by the current Gradle build. Clean Ubuntu installer smoke is
available through `LKJMC_INSTALLER_SMOKE=1 ./scripts/check-installer.sh` and is
skipped by default. The smoke uses an unnamed checkout GID, reruns the installer,
and rejects source ownership, service identity, supplemental-group, secret-mode,
or daemon-process drift.

## Playable mode

With `--playable`, the installer starts the daemon and runs playable bootstrap
after migrations. `bootstrap apply --accept-minecraft-eula` sends the explicit
consent required before writing `eula.txt` or starting Paper or Folia. Read-only
bootstrap plan, status, and doctor operations remain available without consent.

Target flags:

```text
--playable
--accept-minecraft-eula
--bedrock auto|enabled|disabled
--java-port PORT
--bedrock-port PORT
--no-start
```

## Service behavior

The daemon service reads HTTP bearer tokens from a token file, not command-line
text. It uses `RuntimeDirectory=lkjmc`, runs from the checkout so local plugin
assets can be registered, binds daemon HTTP to loopback, and keeps database,
HTTP, and forwarding secrets out of process listings.

## Playable output

Playable success output is compact and truthful: Java address, Bedrock status
pointer, proxy state, hub state, status command, and log command. Optional
degraded features are reported through bootstrap status.
