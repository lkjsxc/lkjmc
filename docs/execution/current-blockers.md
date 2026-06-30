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
- [x] Playable command/menu smoke proves managed token-file daemon auth,
  `/lkjmc`, completion, server list, menus, docs, exchange, and shop item
  delivery when its opt-in prerequisites are accepted.

## Active blockers

1. Complete public wake-and-join controls with expiry, cancellation, cleanup,
   localized states, and Velocity transfer consumption safety.
2. Productize End Expedition through the shop/menu path without duplicate point
   deduction or fake delivery.
3. Add JVM runtime config/schema validation and a deterministic drift check
   against Rust-owned config fields.
4. Add an authenticated private web control surface that delegates mutations to
   daemon commands and audits outcomes.
5. Add a Kubernetes runtime adapter with deterministic manifest planning, real
   object ownership, status, doctor, logs, and guarded live smoke.
6. Expand Docker Compose, source tests, docs drift checks, and opt-in smokes for
   the new surfaces.

## Deferred guardrails

No promoted surface may expose fake success. External live smokes can skip when
credentials, EULA acceptance, Docker, or cluster access are absent, but skipped
checks must not be reported as passed.

## Next executable step

Follow [full implementation pass](tasks/full-implementation-pass.md) and update
`docs/current-state.md` only after each slice is implemented and verified.
