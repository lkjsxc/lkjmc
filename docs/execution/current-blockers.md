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
- [x] Bootstrap root and migration effects, exhaustive effect apply/recording,
  enabled optional-plugin blocking, and richer bootstrap status output.
- [x] Temporary instance and adventure session schema, pure state records,
  allocation planner, transaction-capable store helpers, local daemon
  create/start/stop/get/cleanup commands, cleanup worker, Velocity registration
  hints, daemon-validated Velocity transfer intents, End Expedition purchase,
  startup, refund transaction, `/endexpedition`, menu button, locale keys, and
  permission.

## Active blockers

- [ ] End Expedition: add party selection and return-to-hub behavior.
- [ ] Wake-and-join: add a real queue for suspended backends before enabling
  suspended transfer controls.
- [ ] Control surfaces: keep web and future non-local runtime adapters behind
  documented daemon API seams until real adapters and verification exist.

## Next executable step

Start End Expedition party selection and confirmation menus, backed by the real
daemon purchase command.
