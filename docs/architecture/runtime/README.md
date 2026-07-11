# Runtime architecture

## Purpose

This area owns daemon, CLI, jar registry, and JSON runtime configuration
contracts.


## Status

implemented

## Table of contents

- [Admin RBAC](admin-rbac.md)
- [Adapters](adapters.md)
- [CLI](cli.md)
- [Config](config.md)
- [Connection diagnostics](connection-diagnostics.md)
- [Daemon](daemon/README.md)
- [Discord adapter](discord-adapter.md)
- [Jar registry](jar-registry.md)

## Current and target boundary

The CLI and web use daemon commands for orchestration. Java plugins are
local-safe only and Discord command delegation is withdrawn. Normal CLI
operations do not write PostgreSQL directly except migration and guarded test
reset commands. Pure command planning remains separate from daemon transport,
store, and runtime effects.

## Evidence and degraded behavior

Runtime crates and command tests are source evidence. Unavailable sockets,
tokens, databases, or runtimes return diagnostics; no transport may convert
them into a successful command response.
