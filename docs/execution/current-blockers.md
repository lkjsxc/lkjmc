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

## Active blockers

- [ ] Daemon HTTP auth is source-repaired but not Minecraft-facing smoke-proven.
  Acceptance: prove a managed JVM plugin can read the token file and reach the
  daemon with a mixed-case token, while auth diagnostics stay secret-safe.
- [ ] `/lkjmc` command execution and completion are not field-proven. Acceptance:
  `/lkjmc status`, `/lkjmc doctor`, `/lkjmc server`, and
  `/lkjmc server list` return product output, usage, or daemon diagnostics on
  Paper/Folia and Velocity; completions for `/lkjmc ` and `/lkjmc server ` are
  permission-filtered and product-owned; parser internals never reach users.
- [ ] Dynamic menus are source-mapped but not Minecraft-facing smoke-proven.
  Acceptance: prove `/menu`, server-list loading, one daemon-backed player menu,
  typed diagnostics, and no unintended inventory close in a playable runtime.
- [ ] Playable smoke does not yet prove the reported incident. Acceptance: a
  documented Docker Compose or live smoke with EULA acceptance proves daemon
  auth, `/lkjmc status`, `/lkjmc doctor`, `/lkjmc server`,
  `/lkjmc server list`, completion for `/lkjmc ` and `/lkjmc server `,
  `/menu`, server-list menu loading, and one daemon-backed player menu.

## Deferred guardrails

- Control surfaces: keep web and future non-local runtime adapters behind
  documented daemon API seams until real adapters and verification exist.

## Next executable step

Add and run a playable command/menu smoke with EULA acceptance that joins the
managed network and proves daemon auth, `/lkjmc` output, `/lkjmc` suggestions,
`/menu`, server-list loading, and one daemon-backed player menu through real
Minecraft packets.
