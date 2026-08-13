# Status

## Purpose

Define the current operator status observation and its evidence limits.

## Status

implemented

## Response

`status` is the existing private Unix-socket CLI operation. With PostgreSQL
configured, one bounded SQL statement returns aggregate counts and an ordered
instance view from one database snapshot. At most 32 instances are returned;
the response says when more rows were omitted.

Each instance reports:

- identity and kind;
- desired and latest observed process state;
- tri-state process health, backend readiness, and current proxy registration;
- observation, heartbeat, and registration ages;
- joinability and one exact denial reason; and
- whether bounded diagnostic fields were truncated.

Backend readiness is `null` when the daemon lacks either a process observation
or plugin heartbeat. Velocity readiness is `null` because this view has no
Velocity-aware readiness observation. A stopped backend, stale heartbeat,
missing registration, invalid port, or unhealthy process cannot be joinable.
Status does not refresh runtime state, so it never presents process spawning as
a fresh status effect.

Identifiers and diagnostics have documented character limits in
`instanceSnapshot.fieldCharacterLimits`. The view is deterministic by instance
ID. Human CLI output preserves `unknown` rather than turning missing values into
zero or false and warns when rows were omitted.

The status handler itself performs only the bounded PostgreSQL read. The shared
legacy dispatch layer still records command-completion observability; that
cross-cutting write is not instance or gameplay mutation and remains scheduled
for removal or narrowing with the generic dispatch surface.

A database checkout, query, or deadline failure returns non-success. A source or
database fixture does not prove a real Java process, Minecraft readiness, or a
player path.

## Source owners

- Store query: `crates/lkjmc-store/src/status.rs`.
- Availability policy: `crates/lkjmc-daemon/src/commands/instance_availability.rs`.
- Response: `crates/lkjmc-daemon/src/commands/status_api.rs`.
- CLI rendering: `crates/lkjmc-cli/src/commands_status.rs`.
