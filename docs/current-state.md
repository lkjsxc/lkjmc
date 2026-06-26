# Current state

## Purpose

This ledger states what is implemented now. If it conflicts with any target
contract, this file wins for current behavior.

## Repository and verification

- Documentation topology, line-limit, command-doc, permission-doc, and locale
  catalog checks are implemented.
- `./scripts/verify.sh` runs docs, contract drift checks, Rust
  formatting/lint/tests, daemon/CLI, process runtime, jar registry, installer,
  Minecraft smoke guards, Java tests, and shaded plugin jar assembly.
- Dockerfile and Compose verify scaffolding are implemented.
- Installer and live Minecraft smoke checks are available but opt in because
  they need nested Docker or network/server downloads.

## Rust control plane

- The Cargo workspace contains `lkjmc-core`, `lkjmc-store`, `lkjmc-daemon`,
  `lkjmc-cli`, and `lkjmc-installer` slices.
- `lkjmc-core` has pure models for IDs, instances, jars, players, commands,
  audit events, reconciliation effects, and JSON config validation.
- PostgreSQL migrations create core, instance, jar, player profile, settings,
  sessions, points, homes, warps, parties, achievements, shop, kits, votes,
  teleports, mail, reports, warnings, notes, moderation, daily rewards,
  announcements, commands, audit, and outbox tables.
- `lkjmc-store` applies migrations and provides typed helpers for the tables
  named in [architecture/data/schema.md](architecture/data/schema.md).
- `lkjmc-daemon` serves Unix socket JSON-RPC and a token-protected loopback HTTP
  command endpoint for plugins.
- Daemon command coverage is cataloged in
  [architecture/runtime/daemon/command-catalog.md](architecture/runtime/daemon/command-catalog.md).
- `status` reports daemon start/uptime, database configuration/connectivity,
  PostgreSQL instance/session/jar counts when available, roots, socket path,
  HTTP listener state, and reconciler state.
- `doctor` checks config-file intent, root path syntax, socket parent usability,
  HTTP mode, and database connectivity when configured without printing secrets.
- The daemon loads JSON config, can reload roots and database settings, starts a
  periodic reconciler when a database URL is configured, and recovers stored
  local process observations after daemon restart.
- Local instance orchestration supports create/list/start/stop/restart/delete,
  active-session delete guardrails, bounded logs, explicit launch commands,
  verified jar assets, generated `java -jar` launches, port reservation, and
  template-backed render before launch.
- Jar registry import, PaperMC stable sync, prune, list, inspect,
  checksum verification, and opt-in live PaperMC download smoke are implemented.
- The CLI supports doctor, human and JSON status, config check/reload,
  database migration/status/reset guard, audit tail, verify, jar, instance,
  shop, kit, vote, announcement, player, and moderation families.

## Java and Minecraft adapters

- Java common implements daemon records/client foundation, localization,
  permission constants, menu records, menu reducers, transfer records, and tests.
- Velocity registers `/lkjmc`, `/hub`, server lifecycle commands, `/lkjmc send`,
  reload, restart warning, MOTD, dynamic localhost server registration,
  profile-safe transfer coordination, ban login checks, and tab header/footer.
- Paper/Folia registers the commands listed in
  [product/commands/minecraft.md](product/commands/minecraft.md), uses a
  Folia-aware scheduler bridge, sends heartbeats, opens localized menus, applies
  join-time profiles, records sessions, saves snapshots on quit when configured,
  handles cross-server home/warp/TPA arrivals, enforces chat mutes, and cancels
  scheduled work on disable.
- English and Japanese locale catalogs exist in repository config and Java
  resources with matching key sets.

## Current boundaries

- Template files are read for future renders; running child process directories
  are not rewritten in place.
- Config reload affects new daemon operations; existing child process working
  directories are not rewritten in place.
- Java plugin adapters still parse many daemon JSON bodies through raw strings;
  typed transport is the next plugin hardening target.
- Chunk claims are documented as the next gameplay domain but are not
  implemented yet.
- Live Minecraft smoke automation is implemented but remains opt in.

## Verification status

Default verification is meaningful for docs, pure core, store, daemon API, CLI,
Java common/plugins, local process runtime, and jar registry slices. PostgreSQL
runtime checks run when `LKJMC_STORE_TEST_DATABASE_URL` is set.
