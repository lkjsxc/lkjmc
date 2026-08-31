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
- The configured web bootstrap secret can establish a bounded browser session;
  it is never accepted as TCP command authority or rendered to an adapter.
- Remote command access uses only a PostgreSQL-backed, unexpired scoped `cli`
  or `web` credential. Its scope and the command registry surface are both
  checked before dispatch.
- Generated credentials are written to an owner-limited requested file and
  responses expose only credential id, requested expiry, and fingerprint;
  paths, principals, and scopes are never returned. Withdrawn Java surfaces
  cannot receive credentials.
- The final daemon HTTP address accepts only `127.0.0.1:PORT`; hostnames,
  every other `127/8` address, wildcard, unspecified, IPv6, mapped, and
  zero-port forms fail after CLI overrides.
- Root daemon tokens are never rendered into managed or temporary adapter
  configuration; adapters require separate scoped credentials.
- Bootstrap-secret rotation stages old and new verifiers, proves new-secret
  login over the configured transport before retiring old access, and restores
  the old verifier and file on failure. If restoration fails, it clears both
  in-memory verifiers so the staged secret is rejected; audits contain
  fingerprints only.
- Credential files are synced before their database transaction commits. A
  pre-commit write or audit failure rolls back the credential and attempts
  cleanup without deleting a caller-owned existing file. If cleanup fails, the
  owner-limited file is a recoverable orphan and the response explicitly fails.
  A commit result that is uncertain preserves the file and returns an explicit
  failure, so a database credential is never issued without its file.
- Web session, CSRF, and Kubernetes credentials follow the same redaction and
  owner-limited file rules. Secret-provider failures use fixed redacted denial
  reasons; audits retain no submitted secret, token hash, or secret value.

## Current status

Clean secret provisioning is not a supported installer path. No packaged
installer creates or rewrites PostgreSQL, daemon HTTP, or forwarding secrets.
The Rust EULA policy owner writes only the nonsecret acceptance fact. The immutable update command requires the existing
daemon environment and instance heartbeat credentials to be private and
preserves them without printing or rotating their values. A missing or broadly
readable credential fails preflight before service stop.

Bootstrap and temporary-instance configs retain secret file paths only;
instance creation writes any supplied RCON password to a private file under the
config root before retaining its path. Rendering reads owner-limited files and
creates necessary runtime files privately before their contents are written.
