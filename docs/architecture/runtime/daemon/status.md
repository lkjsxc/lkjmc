# Status and doctor

## Purpose

This document defines the daemon health contract for operators and automation.

## Implemented status body

`status` returns compact JSON with:

- `daemon`, `startedAtUnixSeconds`, and `uptimeSeconds`;
- `database.configured`, `database.connected`, and a sanitized error when a
  configured database cannot be reached or counted;
- `counts.instances`, `counts.activeSessions`, and `counts.jarAssets` when
  PostgreSQL tables are available;
- `roots.config`, `roots.data`, `roots.log`, and `roots.jar`;
- `socket.path`;
- `http.enabled` plus `http.address` when enabled;
- `reconciler.enabled`.

`lkjmc status` prints a human summary by default and preserves the same compact
body with `--json`.

## Implemented doctor checks

`doctor` succeeds only when safe dependency checks pass. It checks that config
loading was intentional, roots are absolute paths with usable ancestors, the
socket parent is a directory, HTTP configuration is enabled or intentionally
disabled, and the configured database can be reached. Database URLs and secrets
are sanitized from errors.

## Current boundary

`status` reports database counts only when migrations have made the tables
available. It does not perform write probes against root directories.

## Source owners

- Dispatch: `crates/lkjmc-daemon/src/api.rs`.
- Status implementation: `crates/lkjmc-daemon/src/status_api.rs`.
- Doctor implementation: `crates/lkjmc-daemon/src/doctor_api.rs`.
- Runtime state: `crates/lkjmc-daemon/src/app.rs`.
- CLI rendering: `crates/lkjmc-cli/src/commands_status.rs`.
