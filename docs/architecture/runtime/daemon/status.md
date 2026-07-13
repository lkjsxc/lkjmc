# Status and doctor

## Purpose

Define the admitted daemon health observation and the rejected diagnostic
boundary.

## Status

implemented

## Status body

`status` is a `postgresql-read` command because its database section reads four
PostgreSQL counts. It returns compact JSON with:

- `daemon`, `startedAtUnixSeconds`, and `uptimeSeconds`;
- sanitized database configuration and one aggregate PostgreSQL count observation;
- configured roots, socket path, and optional HTTP listener;
- selected `runtime.adapter` marked `externalEffects: denied-unproved`;
- `commandLifecycle.admissionLimit`, `deadlineSeconds`, `queue: none`, and
  `externalEffects: denied-unproved`; and
- `reconciler.enabled: false`.

`lkjmc status` prints a human summary by default and preserves the compact body
with `--json`. It observes state only; its four counts are one PostgreSQL
statement, never write probes. A configured database checkout or query failure
is non-success. In particular, `QUERY_CANCELED` and `LOCK_NOT_AVAILABLE`
SQLSTATEs return `command.deadline_exceeded`, never a successful status body.

## Rejected diagnostics

`doctor`, `bootstrap.status`, and every menu-shape command remain registered for
closed contract validation but are `denied-unproved`. They return non-success
before their handlers, filesystem checks, database work, runtime observation,
or bootstrap plan run. This is not a health-success fallback.

## Source owners

- Dispatch: `crates/lkjmc-daemon/src/dispatch.rs`.
- Status: `crates/lkjmc-daemon/src/commands/status_api.rs`.
- Lifecycle: `crates/lkjmc-daemon/src/command_lifecycle.rs`.
- CLI rendering: `crates/lkjmc-cli/src/commands_status.rs`.
