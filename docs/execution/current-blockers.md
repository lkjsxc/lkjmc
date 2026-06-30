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
  server-list command output, one daemon-backed player menu, and no parser leaks.
- [x] The reported `/lkjmc status`, `/lkjmc server`, completion, daemon-auth,
  and dynamic-menu regression is repaired and proven by current playable smoke.
- [x] Admin roles, durable grants, permission mapping, daemon authorization for
  documented admin command families, CLI management, and admin audit rows exist.
- [x] Cobblestone exchange grants exactly one point per block, removes inventory,
  uses idempotent daemon ledger commits, refunds on failure, and has seeded shop
  catalog defaults covered by tests and playable smoke.
- [x] The docs browser, action-bar reducer/events, and dynamic-menu diagnostics
  are implemented and covered by source tests plus playable smoke.
- [x] Adapter-side admin grant snapshot caches feed `/lkjmc` visibility,
  completion, and admin menu enabled states while daemon authorization remains
  final.

## Active blockers

- None.

## Deferred guardrails

- Control surfaces: keep web and future non-local runtime adapters behind
  documented daemon API seams until real adapters and verification exist.

## Next executable step

Reconcile the runtime adapter architecture docs and implementation seams before
adding real web and Kubernetes control surfaces.
