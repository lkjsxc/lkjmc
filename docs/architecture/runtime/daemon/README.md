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

## Current boundaries

Health output is still minimal until the status and doctor target in
[status.md](status.md) is implemented. Command handlers must not claim success
until PostgreSQL or process effects have completed.
