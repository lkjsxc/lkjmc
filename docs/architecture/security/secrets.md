# Secrets

## Purpose

This document defines secret handling rules.


## Status

implemented

## Rules

- PostgreSQL credentials are secrets.
- Generated secrets are stored in owner-limited files with `0600` permissions.
- Instance JSON stores only forwarding and RCON secret-file paths, never their
  contents. RCON uses `rcon.passwordFile`, not `rcon.password`.
- Rendered forwarding and proxy-secret-bearing files use `0600` permissions.
- Secret files are created with `0600` at open time, written and synced before
  publication; no write-then-chmod window is allowed.
- Secrets are not printed after creation.
- The TCP root token is operator-only; adapter access uses bounded scoped
  credentials with an allowed surface, the documented scope allowlist, and
  required expiry.
- Generated credentials are written to an owner-limited requested file and
  responses expose only their path, expiry, and fingerprint.
- The final daemon HTTP address accepts only `127.0.0.1:PORT`; hostnames,
  every other `127/8` address, wildcard, unspecified, IPv6, mapped, and
  zero-port forms fail after CLI overrides.
- Root daemon tokens are never rendered into managed or temporary adapter
  configuration; adapters require separate scoped credentials.
- Token rotation stages old and new verifiers, proves new-token acceptance over
  the configured transport before retiring old access, and restores the old
  verifier and file on failure. If the restoration write fails, it clears both
  in-memory root verifiers so the staged new token is rejected; audits contain
  fingerprints only.
- Web session, CSRF, and Kubernetes credentials follow the same redaction and
  owner-limited file rules.

## Current status

`scripts/install.sh` generates the PostgreSQL password, daemon HTTP token, and
Velocity forwarding secret files under `/etc/lkjmc` or the configured
`LKJMC_CONFIG_ROOT` equivalent with `0600` permissions and never prints generated
values. Restrictive umask changes for installer-owned files are scoped to those
writes so later build outputs stay readable by the daemon service user. The
installer also writes the daemon environment file with `0600` permissions.
Bootstrap and temporary-instance configs retain secret file paths only; instance
creation writes any supplied RCON password to a private file under the config
root before retaining its path. Rendering reads owner-limited files and creates
necessary runtime files privately before their contents are written.
