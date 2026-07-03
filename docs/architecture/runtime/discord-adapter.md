# Discord adapter

## Purpose

This document owns the architecture for the `lkjmc-discord` service.

## Boundaries

The Discord service is a separate Rust process. It does not run inside Velocity,
Paper, Folia, or a Minecraft scheduler thread. It talks to Discord over HTTP or a
Discord gateway library and to lkjmc through the daemon's authenticated command
transport.

## Configuration

The service loads JSON config and validates token sources, daemon transport,
guild allowlist, channel allowlist, role mappings, command registration intent,
interaction bind/public key, and audit actor name before connecting. Secret
values are read from files or environment variables and are never printed.

## Functional core

Pure modules own config validation, command definitions, principal mapping,
role-to-grant evidence, safe diagnostics, account-link command planning,
confirmation payload validation, and daemon request construction. Discord daemon
requests use actor kind `discord` with the Discord user id as actor name.
Adapters own filesystem token reads, Discord REST registration, signed
interaction HTTP, daemon I/O, deferred follow-ups, and process shutdown.
Link-required commands must fail explicitly until the daemon has a durable
Discord-to-Minecraft link.

## Verification

Default tests cover config validation, redaction, command definition shape, and
daemon request construction. Live Discord smoke is opt-in and requires a test bot
token, guild id, allowed channel id, daemon HTTP URL, and daemon bearer token.
