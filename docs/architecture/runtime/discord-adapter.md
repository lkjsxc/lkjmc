# Discord adapter

## Purpose

This document owns the architecture for the `lkjmc-discord` service.


## Status

partial

Missing: server-verified roles, replay and rate-limit persistence, and
server-side confirmation before any Discord action can be enabled.

## Boundaries

The Discord service is a separate Rust process. It does not run inside Velocity,
Paper, Folia, or a Minecraft scheduler thread. Its current boundary is signed
Discord pings and command-list withdrawal; it does not call the daemon.

## Configuration

The service loads JSON config and validates its token source, guild allowlist,
command-withdrawal intent, and interaction bind/public key before connecting.
Secret values are read from files or environment variables and are never printed.

## Functional core

Pure modules own config validation, an empty withdrawal command payload, and
safe diagnostics. Adapters own filesystem token reads, Discord REST replacement
of prior commands, signed interaction HTTP, and process shutdown. Signed
non-ping input denies before any principal mapping or daemon I/O; a mapped role
or a request-body principal is not trusted evidence.

## Verification

Default tests cover config validation, redaction, withdrawal payload shape, and
non-ping denial. The opt-in Discord lane requires a test bot token, application
id, and guild to remove registered commands; absent prerequisites skip or block
the lane and it proves no action authorization.
