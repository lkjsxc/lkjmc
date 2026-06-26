# Status and doctor

## Purpose

This document defines the daemon health contract for operators and automation.

## Current implementation

`status` currently returns a minimal running response. `doctor` currently reports
that the daemon is reachable and whether a database URL is configured.

## Target status body

`status` should return compact JSON with:

- daemon state and process uptime or start timestamp;
- whether a database URL is configured;
- database connectivity result when configured;
- instance, active player session, and jar asset counts when available;
- config, data, log, and jar roots currently in use;
- Unix socket path and HTTP listener enabled or disabled;
- reconciler enabled or disabled.

The CLI should print useful human status by default and preserve compact JSON
with `--json`.

## Target doctor checks

`doctor` should fail when a configured dependency is unusable. It should check
that config loading was intentional, database connectivity works when
configured, roots and socket parent paths are syntactically valid and usable for
the current mode, and no secret values appear in output.

## Source owners

- Dispatch: `crates/lkjmc-daemon/src/api.rs`.
- Runtime state: `crates/lkjmc-daemon/src/app.rs`.
- CLI rendering: `crates/lkjmc-cli/src/commands.rs`.
