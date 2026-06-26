# Install

## Purpose

This document defines current installer behavior and the playable installer
target.

## Target hosts

The host installer targets Ubuntu-like LXC and WSL2 systems where PostgreSQL,
Java 21, Rust, and Gradle can run.

## Current responsibilities

`scripts/install.sh` must be run as root from a checkout. It installs apt
packages, starts PostgreSQL, creates the `lkjmc` user and product roots,
generates the database secret without printing it, creates or updates the
PostgreSQL role and database, writes JSON config, builds and installs Rust
binaries, applies migrations, starts the daemon from that config through systemd
when available, falls back to a WSL-style supervisor command, and runs
`lkjmc doctor`.

The installer is idempotent for directories, user creation, database creation,
secret reuse, config writing, migration application, and service restart. Plugin
jar installation remains limited to artifacts produced by the current Gradle
build. Clean Ubuntu installer smoke is available through
`LKJMC_INSTALLER_SMOKE=1 ./scripts/check-installer.sh` and is skipped by
default.

## Playable target

With `--playable`, the installer starts the daemon and runs playable bootstrap
after migrations. It must require `--accept-minecraft-eula` before writing
`eula.txt` or starting Paper or Folia.

Target flags:

```text
--playable
--accept-minecraft-eula
--bedrock auto|enabled|disabled
--java-port PORT
--bedrock-port PORT
--no-start
```

## Service target

The daemon service should read HTTP bearer tokens from a token file, not command
line text. It should use `RuntimeDirectory=lkjmc`, bind daemon HTTP to loopback,
and keep database, HTTP, and forwarding secrets out of process listings.

## Playable output target

Playable success output is compact and truthful: Java address, Bedrock state,
proxy state, hub state, status command, and log command. Optional degraded
features must be reported as withdrawn, not as failed Java setup.
