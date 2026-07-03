# lkjmc

[![Verify](https://github.com/lkjsxc/lkjmc/actions/workflows/verify.yml/badge.svg)](https://github.com/lkjsxc/lkjmc/actions/workflows/verify.yml)

## Purpose

`lkjmc` is a server-side Minecraft network control plane for Ubuntu-like LXC
and WSL2 hosts. The current project uses Rust for the daemon, CLI, store, and
local orchestration; Java 21 for Velocity and Paper/Folia adapters; and
PostgreSQL as durable truth.

## Start here

- [Coding-agent entry point](AGENTS.md)
- [Documentation index](docs/README.md)
- [Implemented state](docs/state/README.md)
- [Current blockers](docs/execution/current-blockers.md)

## Current status

The repository is beyond scaffolding. It includes the PostgreSQL schema and
store helpers, daemon and CLI command surfaces, local-process and Kubernetes
runtime adapters, jar registry, installer slice, web control pages, Discord
adapter, Java common contracts, Velocity and Paper/Folia plugins, GUI framework,
profile sync, moderation, mail, kits, votes, daily rewards, announcements, and
verification gates.

Treat [docs/state/README.md](docs/state/README.md) as the authoritative
ledger for shipped behavior. Product and architecture docs define owner
contracts and may also name the next target boundaries.

## Local checks

```sh
./scripts/check-lines.py
./scripts/check-docs.py
./scripts/verify-full.sh
```

Each successful quiet check prints one bounded success line.
