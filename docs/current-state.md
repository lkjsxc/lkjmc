# Current state

## Purpose

This ledger states what is implemented now. If it conflicts with any target
contract, this file wins for current behavior.

## Repository and verification

- Documentation topology, line-limit, bootstrap-doc, asset-doc, command-doc,
  permission-doc, and locale catalog checks are implemented.
- `./scripts/verify.sh` runs docs, contract drift checks, Rust
  formatting/lint/tests, daemon/CLI, process runtime, jar registry, installer,
  Minecraft smoke guards, Java tests, and shaded plugin jar assembly.
- Dockerfile and Compose verify scaffolding are implemented.
- Daemon tests cover claim create/trust/list/snapshot/delete dispatch.
- Opt-in claim smoke coverage starts the daemon, creates/trusts/lists/deletes a
  claim through PostgreSQL and CLI surfaces, and verifies snapshots.
- Opt-in live Paper claim smoke starts a real Paper jar and daemon HTTP API,
  then the Paper plugin creates, trusts, snapshots, decides, and deletes a claim.
- An additional opt-in claim protocol smoke joins Paper as real offline-mode
  players, issues `/claim`, and sends break/place packets against the claim.
- Installer and live Minecraft smoke checks are available but opt in because
  they need nested Docker or network/server downloads.

## Rust control plane

- The Cargo workspace contains `lkjmc-core`, `lkjmc-store`, `lkjmc-daemon`,
  `lkjmc-cli`, and `lkjmc-installer` slices.
- `lkjmc-core` has pure models for IDs, instances, jars, players, commands,
  audit events, reconciliation effects, playable bootstrap planning, and JSON
  config validation.
- PostgreSQL migrations create core, instance, jar, generic asset, plugin
  installation, bootstrap run, player profile, settings, sessions, points, homes,
  warps, parties, achievements, shop, kits, votes, teleports, mail, reports,
  warnings, notes, moderation, daily rewards, announcements, chunk claims,
  commands, audit, and outbox tables.
- `lkjmc-store` applies migrations and provides typed helpers for the tables
  named in [architecture/data/schema.md](architecture/data/schema.md), including
  assets, plugin installations, and bootstrap run ledgers.
- `lkjmc-daemon` serves Unix socket JSON-RPC and a token-protected loopback HTTP
  command endpoint for plugins.
- `lkjmc-daemon` serves claim create/delete/list/snapshot/trust/untrust commands
  backed by PostgreSQL and audit events.
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
  claim list/delete, shop, kit, vote, announcement, player, and moderation
  families.

## Java and Minecraft adapters

- Java common implements daemon records/client foundation, Gson-backed typed
  daemon JSON transport, localization, permission constants, menu records, menu
  reducers, transfer records, and tests.
- Velocity registers `/lkjmc`, `/hub`, server lifecycle commands, `/lkjmc send`,
  reload, restart warning, MOTD, dynamic localhost server registration,
  profile-safe transfer coordination, ban login checks, and tab header/footer.
- Paper/Folia registers the commands listed in
  [product/commands/minecraft.md](product/commands/minecraft.md), uses a
  Folia-aware scheduler bridge, sends heartbeats, opens localized menus, applies
  join-time profiles, records sessions, saves snapshots on quit when configured,
  handles cross-server home/warp/TPA arrivals, enforces chat mutes, protects
  known claimed chunks from an immutable async snapshot, and cancels scheduled
  work on disable.
- English and Japanese locale catalogs exist in repository config and Java
  resources with matching key sets.

## Current boundaries

- Template files are read for future renders; running child process directories
  are not rewritten in place.
- Config reload affects new daemon operations; existing child process working
  directories are not rewritten in place.
- Java plugin adapters consume typed daemon JSON response bodies through common
  helpers instead of raw body string searches.
- Chunk claims are implemented for one-chunk creation, listing, deletion,
  trust, untrust, here inspection, async snapshot refresh, pure break/place/basic
  interact decisions, and Paper protection listeners. During daemon outage,
  known claimed chunks stay protected from the last snapshot and unknown chunks
  are allowed.
- Live Minecraft and live Paper claim smoke automation are implemented but
  remain opt in.

## Verification status

Default verification is meaningful for docs, pure core, store, daemon API, CLI,
Java common/plugins, local process runtime, and jar registry slices. PostgreSQL
runtime checks run when `LKJMC_STORE_TEST_DATABASE_URL` is set.
