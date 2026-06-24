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

The installer is not implemented. `scripts/install.sh` exits with a clear
failure message.
