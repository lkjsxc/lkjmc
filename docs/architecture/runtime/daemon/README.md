# Daemon

## Purpose

This area owns daemon transport, health, and public command catalog contracts.

## Table of contents

- [Command catalog](command-catalog.md)
- [Status and doctor](status.md)
- [Transport](transport.md)

## Implemented responsibilities

`lkjmc-daemon` serves newline-delimited Unix socket JSON-RPC and a
bearer-token loopback HTTP command endpoint. It loads JSON config, reads the
PostgreSQL secret file without printing it, owns local process orchestration,
uses the store for durable state, and exposes the command catalog.

## Source owners

- Dispatch root: `crates/lkjmc-daemon/src/api.rs`.
- Instance router: `crates/lkjmc-daemon/src/instance_api.rs`.
- HTTP transport: `crates/lkjmc-daemon/src/http_api.rs`.
- Unix socket transport: `crates/lkjmc-daemon/src/socket_api.rs`.
- Runtime state: `crates/lkjmc-daemon/src/app.rs`.

## Health contract

The status and doctor contract in [status.md](status.md) is implemented for
operator use. Health output must stay aligned with the current-state ledger when
new PostgreSQL, runtime, bootstrap, or transport checks are added.

## Truthfulness rule

Command handlers must not claim success until PostgreSQL, filesystem, network,
probe, or process effects have completed. Unsupported effects fail explicitly
instead of falling through to success.
