# Secrets

## Purpose

This document defines secret handling rules.


## Status

implemented

## Rules

- PostgreSQL credentials are secrets.
- Generated secrets are stored in owner-limited files with `0600` permissions.
- Instance JSON stores only a forwarding-secret file path, never its contents.
- Rendered forwarding and proxy-secret-bearing files use `0600` permissions.
- Secret files are created with `0600` at open time, written and synced before
  publication; no write-then-chmod window is allowed.
- Secrets are not printed after creation.
- The TCP root token is operator-only; adapter access uses bounded scoped
  credentials with an allowed surface, allowlisted scopes, and required expiry.
- Generated credentials are written to an owner-limited requested file and
  responses expose only their path, expiry, and fingerprint.
- Daemon HTTP accepts only literal loopback socket addresses.
- Token rotation writes replacement files atomically, proves new acceptance and
  old rejection through the live configured transport, and audits fingerprints.
- Web session, CSRF, and Kubernetes credentials follow the same redaction and
  owner-limited file rules.

## Current status

`scripts/install.sh` generates the PostgreSQL password, daemon HTTP token, and
Velocity forwarding secret files under `/etc/lkjmc` or the configured
`LKJMC_CONFIG_ROOT` equivalent with `0600` permissions and never prints generated
values. Restrictive umask changes for installer-owned files are scoped to those
writes so later build outputs stay readable by the daemon service user. The
installer also writes the daemon environment file with `0600` permissions.
Bootstrap and temporary-instance configs retain secret file paths only; rendering
reads those owner-limited files and creates necessary runtime files privately
before their contents are written.
