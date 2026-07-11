# Execution view

## Purpose

This view traces command execution from ingress through durable and external
work.

## Status

implemented

## Execution path

CLI, JVM, Discord, and web adapters submit a command envelope. Axum transport
validates size and authentication, then uses `spawn_blocking` before synchronous
dispatch. Dispatch selects a command handler. The handler validates, authorizes,
uses PostgreSQL helpers, and calls a runtime, asset, or bootstrap adapter only
when required. Responses return success only after the handler's required work
reports success.

JVM callbacks do not block scheduler threads on daemon, database, filesystem,
network, or process work; their adapters move that work off the scheduler.

## Exact non-atomic boundaries

- HTTP receipt and command execution are not one transaction; a client timeout
  does not prove that a started command was cancelled.
- `instance.restart` stops and starts through distinct runtime calls before its
  desired-state update.
- A command response and a later Java UI render are separate effects.

## Source trace

- `crates/lkjmc-daemon/src/transport/command.rs`
- `crates/lkjmc-daemon/src/dispatch.rs`
- `crates/lkjmc-daemon/src/commands/instance_lifecycle.rs`
- `crates/lkjmc-daemon/src/runtime/adapter.rs`
- `platforms/jvm/paper/src/main/java/com/lkjmc/paper/SchedulerBridge.java`
