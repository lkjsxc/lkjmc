# Runtime architecture

## Purpose

This area owns daemon, CLI, jar registry, and JSON runtime configuration
contracts.

## Table of contents

- [Admin RBAC](admin-rbac.md)
- [CLI](cli.md)
- [Config](config.md)
- [Connection diagnostics](connection-diagnostics.md)
- [Daemon](daemon/README.md)
- [Jar registry](jar-registry.md)

## Contract

The CLI and plugins use the daemon API for orchestration. Normal CLI operations
do not write directly to PostgreSQL except migration and guarded test reset
commands.
