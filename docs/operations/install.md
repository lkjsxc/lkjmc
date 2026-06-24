# Install

## Purpose

This document defines the target installer contract.

## Target hosts

- Ubuntu LXC with systemd
- WSL2 Ubuntu with or without systemd

## Responsibilities

The installer will install Java 21, PostgreSQL, build dependencies, create the
`lkjmc` user and roots, initialize the database, write JSON config, build or
install binaries and plugin jars, apply migrations, start the daemon, and run
`lkjmc doctor`.

## Current status

`scripts/install.sh` implements the first Ubuntu/WSL installer slice. It must be
run as root from a checkout. It installs apt packages, starts PostgreSQL, creates
the `lkjmc` user and roots, generates the database secret without printing it,
creates or updates the PostgreSQL role and database, writes JSON config, builds
and installs the Rust binaries, applies migrations, starts the daemon from that
config through systemd when available, falls back to a local WSL-style
supervisor command, and runs `lkjmc doctor`.

The installer is idempotent for directories, user creation, database creation,
secret reuse, config writing, migration application, and service restart. Plugin jar installation remains limited to whatever the current Gradle build
produces. Clean Ubuntu installer smoke is available through
`LKJMC_INSTALLER_SMOKE=1 ./scripts/check-installer.sh` and is skipped by default.
