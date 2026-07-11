# Current blockers

## Purpose

This document lists the next executable blockers in priority order.

## Completed foundation

- [x] Repository, build, core model, config, store, daemon API, and CLI.
- [x] Local process runtime, jar registry, installer scripts, and Java common.
- [x] Velocity plugin, Paper/Folia plugin foundation, GUI framework, and
  player profile sync.
- [x] Documentation topology, line-limit, command, permission, locale, asset,
  bootstrap, and smoke guard checks.
- [x] Admin RBAC, daemon authorization, audit, chunk claims, menu runtime,
  exchange, shop item delivery, and temporary End command flow.
- [x] Runtime orchestration has an adapter boundary with `local-process` behind
  the seam and status/doctor reporting.
- [x] Daemon HTTP token rotation has daemon commands, CLI commands, atomic file
  replacement, auth hot-swap, Java token-file reload, and old/new token tests.
- [x] Public wake-and-join controls have durable request, status, cancellation,
  expiry cleanup, consume, menu wake action, and Velocity transfer safety paths.
- [x] End Expedition shop delivery delegates to the daemon adventure purchase
  path, records shop purchase after success, and avoids duplicate point spend.
- [x] JVM runtime config/schema validation and Rust-to-Java drift checks exist.
- [x] Private web control pages delegate reads and mutations to daemon commands,
  support browser login, session cookies, CSRF-protected forms, bearer-safe API
  paths, and guarded smoke coverage.
- [x] Kubernetes runtime config, manifest planner, `kubectl` effect adapter,
  typed pod observation, logs, stop, recover, delete, and guarded smoke guidance
  exist.
- [x] Promoted docs drift checks, config schema checks, web smoke, and
  Kubernetes smoke are wired into verification with skip-by-default behavior.
- [x] Playable command/menu smoke proves managed token-file daemon auth,
  `/lkjmc`, completion, server list, menus, docs, exchange, and shop item
  delivery when its opt-in prerequisites are accepted.
- [x] The Discord adapter validates credentials, registers real slash-command
  metadata, verifies signed interaction HTTP requests, delegates supported
  commands to daemon HTTP, maps principals and roles, redacts diagnostics, and
  has guarded smoke coverage.
- [x] GitHub Actions CI runs the full Docker Compose verify gate on pushes to
  `main` and pull requests.
- [x] Daemon runtime database access uses a real PostgreSQL pool with
  configurable `database.poolSize`; multi-write command helpers can run inside a
  caller-owned transaction.
- [x] Daemon and Discord inbound transports use axum; CLI commands use HTTP
  `POST /command` over the daemon Unix socket.
- [x] `contracts/commands.json` is the daemon command registry consumed by Rust
  dispatch tests, Java command target tests, and command documentation checks.
- [x] The daemon source root is split into domain directories with thin root
  files and no remaining `player_*_api` modules.
- [x] Locale catalogs have one committed source under `config/locales/`, Gson
  parsing, build-time JVM bundling, and slim parity/reference checks.
- [x] Menu server transfer rows emit real save-first Velocity transfers with
  localized sending and failure feedback.
- [x] Discord account linking has hashed one-time codes, daemon complete/remove
  commands, and Discord slash-command planning for link/unlink.
- [x] The unused outbox store module is removed and a migration drops the
  write-only table; recovery reports now state their record-only semantics.
- [x] Store integration tests use per-test PostgreSQL schemas and CLI parsing
  has Rust unit coverage.
- [x] Verification is split into fast/full/live scripts, Compose uses profiles,
  and smoke harness sources live under `tests/smoke/`.
- [x] State is split under `docs/state/`, owner docs have checked statuses, and
  closed execution tasks are archived.
- [x] Final deterministic acceptance gates are green and defect register
  dispositions are recorded.
- [x] Menu structure is JSON contract data, checked by `check-menus.py`, served
  by the JVM menu engine, and no previous menu framework references remain.
- [x] The updated playable command/menu smoke passed with EULA acceptance,
  proving the engine-backed menu, docs, shop, exchange, and settings paths.

## Active blockers

- [ ] Complete `D-INVENTORY` from the active
  [autonomous evolution ledger](tasks/autonomous-evolution.md): inventory every
  documentation file, define evidence dimensions, and record contradictions
  before any implementation change.
- [ ] Repair the generated-artifact line-check defect: nested Gradle output can
  fail `check-lines.py` after a build even though clean pre-build checks pass.
  This remains scheduled behind the documentation barrier.
- [ ] Remaining live-tier smoke execution when exact credentials, legal EULA
  acceptance, endpoints, or cluster access become available.

## Deferred guardrails

No promoted surface may expose fake success. External live smokes can skip only
with an exact missing prerequisite and must never be reported as passed.

## Live prerequisites

- Minecraft: `LKJMC_MINECRAFT_SMOKE=1`.
- Minecraft claim: `LKJMC_MINECRAFT_CLAIM_SMOKE=1`.
- Playable: `LKJMC_PLAYABLE_SMOKE=1` and `LKJMC_ACCEPT_MINECRAFT_EULA=1`.
- Bedrock: `LKJMC_BEDROCK_SMOKE=1` plus a supported endpoint and client.
- Discord: `LKJMC_DISCORD_SMOKE=1` plus real credentials and interaction access.
- Kubernetes: `LKJMC_KUBERNETES_SMOKE=1`, `kubectl`, and an authorized
  disposable namespace.

## Next executable step

Claim `D-INVENTORY` through
`tmp/lkjmc-autonomous-evolution-plan/control/planctl.py` and complete the
repository documentation coverage ledger.
