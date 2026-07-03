# AGENTS

## Purpose

This is the entry point for coding agents working on `lkjmc`. The repository is
a docs-first Minecraft network control plane: PostgreSQL stores durable truth,
Rust owns daemon/CLI/store/orchestration, and Java 21 owns Velocity and
Paper/Folia adapters.

## Non-negotiable rules

- Update owner docs before changing behavior.
- Keep authored Markdown and source files at or below 200 lines.
- Use JSON for user-edited configuration.
- Use PostgreSQL as the product data store.
- Keep pure cores separate from adapters that perform effects.
- Never register fake commands, fake sync, fake process management, or fake
  downloads.
- Do not block Minecraft scheduler threads on database, filesystem, network, or
  process work.
- Do not print generated secrets.
- Keep generated files, jars, logs, databases, and `tmp/` out of commits.

## Read order

1. [docs/state/README.md](docs/state/README.md)
2. [docs/agent/README.md](docs/agent/README.md)
3. [docs/execution/current-blockers.md](docs/execution/current-blockers.md)
4. The README for the area being changed
5. The exact owner doc for the behavior being changed

## Task routing

- If the user names a task, do it.
- Otherwise take the first incomplete blocker in
  [docs/execution/current-blockers.md](docs/execution/current-blockers.md).
- Commit broad docs-only contract changes before dependent implementation
  changes.
- Keep [docs/state/README.md](docs/state/README.md) aligned with shipped
  behavior.

## Verification gates

Run the narrowest relevant checks while working. Before handoff, run every gate
that is available for the touched area and report exact commands and results.
The fast local checks and the full Compose gate are:

```sh
./scripts/check-lines.py
./scripts/check-docs.py
./scripts/verify-full.sh
docker compose --profile verify run --rm verify
```

## Handoff requirements

Every handoff must include summary, docs changed, implementation changed,
verification, not-tested items, risks, and one next executable step. Never claim
an unrun gate passed.
