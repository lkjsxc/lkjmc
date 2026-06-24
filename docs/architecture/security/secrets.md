# Secrets

## Purpose

This document defines secret handling rules.

## Rules

- PostgreSQL credentials are secrets.
- Generated secrets are stored in root-readable files with `0600` permissions.
- Secrets are not printed after creation.
- Plugin HTTP uses a shared local token unless a stronger local mechanism is
  implemented.
- Daemon HTTP listens on loopback by default.

## Current status

Secret generation is not implemented yet.
