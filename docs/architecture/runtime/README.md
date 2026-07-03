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

## Contract

The CLI and plugins use the daemon API for orchestration. Normal CLI operations
do not write directly to PostgreSQL except migration and guarded test reset
commands.
