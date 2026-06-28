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

## Active blockers

- [ ] Bootstrap truthfulness: root and migration effects must be planned,
  applied, and recorded without a catch-all success path.
- [ ] Temporary instances: add durable data, runtime allocation, readiness,
  retention, and cleanup before live adventure purchases.
- [ ] End Expedition: make purchase, points deduction, temporary instance
  creation, startup failure refund, transfer, and audit one daemon-owned flow.
- [ ] Wake-and-join: add a real queue for suspended backends before enabling
  suspended transfer controls.
- [ ] Control surfaces: keep web and future non-local runtime adapters behind
  documented daemon API seams until real adapters and verification exist.

## Next executable step

Finish the bootstrap truthfulness blocker, then run the docs and Rust bootstrap
checks before starting temporary-instance schema work.
