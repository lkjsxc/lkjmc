# Secrets

## Purpose

This document defines secret handling rules.


## Status

implemented

## Rules

- PostgreSQL credentials are secrets.
- Generated secrets are stored in owner-limited files with `0600` permissions.
- Secret-file creation must scope restrictive umask changes to the secret write.
- Secrets are not printed after creation.
- Plugin HTTP uses a shared local token unless a stronger local mechanism is
  implemented.
- Daemon HTTP listens on loopback by default.
- Token rotation writes replacement files atomically and audits only
  non-reversible fingerprints.
- Web session, CSRF, and Kubernetes credentials follow the same redaction and
  owner-limited file rules.

## Current status

`scripts/install.sh` generates the PostgreSQL password, daemon HTTP token, and
Velocity forwarding secret files under `/etc/lkjmc` or the configured
`LKJMC_CONFIG_ROOT` equivalent with `0600` permissions and never prints generated
values. Restrictive umask changes are scoped to secret and environment-file
writes so later build outputs stay readable by the daemon service user. The
installer also writes the daemon environment file with `0600` permissions.
