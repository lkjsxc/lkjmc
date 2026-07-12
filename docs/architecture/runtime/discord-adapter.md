# Discord adapter

## Purpose

This document owns the architecture for the `lkjmc-discord` service.


## Status

partial

Missing: server-verified roles, replay and rate-limit persistence, and
server-side confirmation before any Discord action can be enabled.

## Boundaries

The Discord service is a separate Rust process. It does not run inside Velocity,
Paper, Folia, or a Minecraft scheduler thread. Its only executable boundary is
command-list withdrawal; it does not call the daemon or start an interaction
listener. `interactionBind` is refused before token read, listener bind, or REST
work because no trusted admission, worker tracking, or shutdown boundary exists.

## Configuration

The service loads JSON config and validates its token source, guild allowlist,
and command-withdrawal intent. An interaction bind is an explicit startup error,
not an enabled endpoint. Secret values are read from files or environment
variables and are never printed.

## Functional core

Pure modules own config validation, an empty withdrawal command payload, and
safe diagnostics. Adapters own filesystem token reads and Discord REST
replacement of prior commands. No request body, signature, role, or principal
reaches the process; Discord action authorization remains unavailable.

## Verification

Default tests cover bind withdrawal, config redaction, and withdrawal payload
shape. The opt-in Discord lane requires a test bot token, application id, and
guild to remove registered commands; absent prerequisites skip or block the lane
and it proves no action authorization or interaction service.
