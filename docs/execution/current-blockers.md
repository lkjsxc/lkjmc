# Current blockers

## Purpose

This document lists the next executable blockers in priority order.

## Completed foundation

- [x] Repository, build, core model, config, store, daemon API, and CLI.
- [x] Local process runtime, jar registry, installer, and Java common module.
- [x] Velocity plugin, Paper/Folia plugin foundation, GUI framework, and
  player profile sync.
- [x] SMP and proxy utility imports, moderation, mail, kits, votes, daily
  rewards, announcements, points leaderboard, and live smoke scaffolding.
- [x] Documentation truth refresh and contract drift checks for commands,
  permissions, and locale catalogs.
- [x] Real daemon status and doctor checks.
- [x] Typed Java daemon transport and adapter JSON helpers.
- [x] PostgreSQL-backed chunk claims with Paper/Folia command and protection.
- [x] Opt-in daemon/CLI claim smoke coverage.
- [x] Opt-in live Paper claim integration smoke coverage.
- [x] Opt-in protocol-level claim command and block packet smoke coverage.
- [x] Bootstrap truthfulness, temporary instance/adventure lifecycle, Velocity
  registration hints, transfer intents, End Expedition flow, and wake-and-join.
- [x] Source-level shared `/lkjmc` command model, adapter completion wiring,
  typed dynamic-menu diagnostics, no-close ordinary menu effects, and
  nether-star hotbar token tests.
- [x] Playable command/menu smoke proves managed token-file daemon auth with a
  mixed-case token, Velocity Brigadier `/lkjmc` output and suggestions, `/menu`,
  server-list data, one daemon-backed player menu, and no parser leaks.

## Active blockers

- [ ] Reproduce and repair the reported `/lkjmc status`, `/lkjmc server`,
  completion, command-output, daemon-auth, and dynamic-menu regression. Do not
  close this blocker until source tests and a current playable smoke prove the
  exact player-facing surfaces.
- [ ] Implement coherent admin roles, grants, permission mapping, daemon
  enforcement, CLI operations, Minecraft visibility, and audit trails.
- [ ] Implement cobblestone exchange at exactly one point per block, safe
  inventory removal, idempotent daemon ledger grant, refund-on-failure, and
  tested shop catalog defaults.
- [ ] Implement the in-game docs browser, action-bar reducer, and dynamic menu
  diagnostic polish described in the owner docs.

## Deferred guardrails

- Control surfaces: keep web and future non-local runtime adapters behind
  documented daemon API seams until real adapters and verification exist.

## Next executable step

Strengthen reproduction tests and playable smoke assertions for `/lkjmc status`,
`/lkjmc server`, root and server completions, daemon auth, and dynamic menu
diagnostics, then repair the shared command runtime.
