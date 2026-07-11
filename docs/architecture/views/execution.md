# Execution view

## Purpose

This view traces command execution from ingress through durable and external
work.

## Status

implemented

## Execution path

CLI and web adapters submit a command envelope. Axum transport validates size
and authentication, then uses `spawn_blocking` before synchronous dispatch.
Dispatch selects a command handler. The handler validates, authorizes, uses
PostgreSQL helpers, and calls a runtime, asset, or bootstrap adapter only when
required. Responses return success only after the handler's required work
reports success.

Java plugins are local-safe presentation only and submit no daemon envelope
pending trusted identity/session attestation.

## Exact non-atomic boundaries

- HTTP receipt and command execution are not one transaction; a client timeout
  does not prove that a started command was cancelled.
- `instance.restart` stops and starts through distinct runtime calls before its
  desired-state update.
- A command response and a later web render are separate effects.

## Source trace

- `crates/lkjmc-daemon/src/transport/command.rs`
- `crates/lkjmc-daemon/src/dispatch.rs`
- `crates/lkjmc-daemon/src/commands/instance_lifecycle.rs`
- `crates/lkjmc-daemon/src/runtime/adapter.rs`
- `crates/lkjmc-daemon/src/web/routes.rs`
