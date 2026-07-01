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
- [x] Private web control pages delegate reads and mutations to daemon commands
  under bearer authentication.
- [x] Kubernetes runtime config, manifest planner, `kubectl` effect adapter,
  raw pod status, logs, stop, delete, and guarded smoke guidance exist.
- [x] Promoted docs drift checks, config schema checks, web smoke, and
  Kubernetes smoke are wired into verification with skip-by-default behavior.
- [x] Playable command/menu smoke proves managed token-file daemon auth,
  `/lkjmc`, completion, server list, menus, docs, exchange, and shop item
  delivery when its opt-in prerequisites are accepted.

## Active blockers

1. Complete web browser control hardening: login, session cookies, CSRF for
   form posts, bearer-safe API paths, redacted rendering, tests, and smoke
   coverage.
2. Complete Kubernetes runtime observation and recovery safety with typed pod
   JSON parsing, bounded logs, exact owned-label destructive operations, tests,
   and guarded live smoke diagnostics.
3. Complete the Discord adapter as a live daemon-backed command adapter with
   real command registration/handling paths, principal mapping, redacted
   diagnostics, tests, and guarded live smoke coverage.

## Deferred guardrails

No promoted surface may expose fake success. External live smokes can skip when
credentials, EULA acceptance, Docker, or cluster access are absent, but skipped
checks must not be reported as passed.

## Next executable step

Start with the web control hardening blocker, then update this queue after its
owner docs, source, tests, and smoke script agree.
