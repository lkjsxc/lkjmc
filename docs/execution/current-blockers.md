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

## Active blockers

- [ ] Refresh documentation truth maps and add drift checks for commands,
  permissions, and locale catalogs.
- [ ] Replace static daemon health output with real status and doctor checks.
- [ ] Replace raw Java daemon body parsing with typed JSON transport helpers.
- [ ] Implement PostgreSQL-backed chunk claims after the claim contracts are in
  place.

## Next executable step

Finish the documentation refresh and contract drift checks, then implement real
daemon status and doctor output.
