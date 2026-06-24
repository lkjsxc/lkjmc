# lkjmc

## Purpose

`lkjmc` is being built as a server-side Minecraft network control plane for
Ubuntu-like LXC and WSL2 hosts. The target product has one daemon, one CLI,
one PostgreSQL store, one Velocity plugin, one Paper/Folia plugin, and a shared
JSON contract.

## Start here

- [Coding-agent entry point](AGENTS.md)
- [Documentation index](docs/README.md)
- [Current implemented state](docs/current-state.md)
- [Current blockers](docs/execution/current-blockers.md)

## Current status

Only the repository documentation and repository checks are implemented. Runtime
services, database schema, installer, CLI, and plugins are not implemented yet.
See [docs/current-state.md](docs/current-state.md) for the authoritative ledger.

## Local checks

```sh
./scripts/check-lines.py
./scripts/check-docs.py
./scripts/verify.sh
```

Each successful quiet check prints one bounded success line.
