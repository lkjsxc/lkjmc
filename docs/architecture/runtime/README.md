# Runtime architecture

## Purpose

This area owns daemon, CLI, and JSON runtime configuration contracts.

## Table of contents

- [CLI](cli.md)
- [Config](config.md)
- [Daemon](daemon.md)

## Contract

The CLI and plugins use the daemon API for orchestration. Normal CLI operations
do not write directly to PostgreSQL.
