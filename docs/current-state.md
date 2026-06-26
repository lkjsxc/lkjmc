# Current state

## Purpose

This ledger states what is implemented now. If it conflicts with any target
contract, this file wins for current behavior.

## Implemented

- Repository documentation skeleton is implemented.
- Line-limit and documentation topology checks are implemented.
- Cargo workspace scaffolding is implemented for five Rust crates.
- Gradle multiproject scaffolding is implemented for Java common, Velocity, and
  Paper/Folia modules.
- Dockerfile, Compose verify scaffolding, opt-in live Minecraft smoke
  automation, and opt-in banned-player proxy login smoke are implemented.
- `scripts/verify.sh` runs docs, Rust, Java tests, shaded plugin jar assembly,
  store, daemon/CLI, and process runtime checks.
- `lkjmc-core` has pure Rust models for IDs, instances, jars, players,
  commands, audit events, and reconciliation effects.
- `lkjmc-core` parses and validates main and instance JSON config strings.
- PostgreSQL migrations create the current core schema foundation, including
  party invites.
- `lkjmc-store` applies migrations and provides typed insert/read helpers for
  nodes, instances, jars, player profile records, player snapshot leases,
  player settings, active sessions, points accounts, homes, warps, parties,
  achievements, shop items/purchases, kits, vote links/rewards, pending teleports,
  player mail, player report review/close, player warnings, player notes, moderation bans,
  daily rewards, announcements, commands, audit, and outbox.
- `lkjmc-daemon` serves Unix socket JSON-RPC for `doctor`, `status`,
  `audit.tail`, player profile inspect/load/snapshot/restore commands, player settings,
  points balance, active session join/leave, server-local homes/warps, player
  mail, kits, vote links/rewards, player report review/close, player warnings, player notes,
  moderation bans/status, daily rewards, announcements, and audit-backed player transfer/recovery event commands.
- `lkjmc-daemon` has a token-protected loopback HTTP command endpoint and can
  load and reload daemon roots and database connection settings from JSON config.
- `lkjmc-daemon` can start, stop, restart, observe, delete, and tail logs for
  instances that have an explicit local launch command in their JSON config.
- `lkjmc-daemon` runs a periodic reconciler for explicit launch-command
  instances when a database URL is configured.
- The local runtime writes bounded process output under the configured log root,
  records observations in PostgreSQL, and recovers live process-group handles
  from stored observations after daemon restart.
- `lkjmc` CLI supports `doctor`, `status`, `config check`, `config reload`, `db migrate`,
  `db status`, `audit tail`, `verify`, moderation reports/report close/warn/note/ban/unban/status,
  shop, kit, and vote link/reward administration, announcements, player inspect/snapshot/restore, and the current instance
  list/create/start/stop/restart/delete/log commands.
- Instance delete refuses active player sessions recorded in PostgreSQL unless
  `--force` is supplied.
- Jar registry import, PaperMC stable sync, prune, list, inspect,
  launch-time checksum verification, and opt-in live PaperMC download smoke are
  implemented.
- Instances may launch from a verified `jarAssetId` with a generated
  `java -jar` command.
- The daemon allocates or reserves a server port in PostgreSQL, renders
  template-backed instance directories before launch, and runs processes with
  that directory as the working directory.
- Stop attempts configured RCON `stop`, writes `stop` to process stdin when
  available, and then uses process-group signal escalation.
- `scripts/install.sh` implements the first idempotent Ubuntu/WSL checkout
  installer slice, generates the database secret without printing it, and
  `scripts/check-installer.sh` provides an opt-in clean Ubuntu installer smoke.
- Java common implements initial platform-neutral daemon records, localization,
  permission constants, menu records, menu click decisions, and tests.
- Velocity module builds a plugin jar that registers `/lkjmc status`, `/lkjmc
  server list`, daemon-backed server lifecycle commands, `/lkjmc send`,
  `/lkjmc reload`, `/lkjmc restart warn`, `/hub`, MOTD handling, dynamic
  localhost server registration, profile-safe transfer save acknowledgements,
  login ban checks, and post-login tab header/footer handling.
- Paper/Folia module builds a plugin jar with lifecycle, Folia scheduler bridge,
  `/lkjmc status`, `/menu`, `/lang <en|ja>`, `/points`, `/sethome`, `/home`,
  `/lkjmc server ...`, `/setwarp`, `/warp`, `/tpa`, `/tpaccept`, `/party`,
  `/achievements`, `/hud`, `/shop`, `/buy`, `/kit`, `/vote`, `/mail`, `/report`, `/reports`,
  `/warn`, `/warnings`, `/note`, `/notes`, `/ban`, `/unban`, `/daily`, `/announce`,
  cross-server home/warp/TPA bridge teleports, join/home/shop achievement triggers,
  periodic action-bar HUD refresh when enabled,
  localization-backed root/server/settings/language menu contracts, pagination
  and confirmation
  menu contracts, hotbar menu
  entrypoint guardrails, daemon-backed heartbeat/status, join-time profile
  apply, join/quit active session records, save-on-quit profile snapshots when
  configured, and task cancellation
  on disable.

## Not implemented

- Template files are read at each instance render, so edits apply to future
  renders without a daemon restart; running child process directories are not
  rewritten in place.
- Live Minecraft jar download smoke is implemented but remains opt-in and is not
  part of default verify yet.
- Live Minecraft smoke automation starts standalone Paper and Velocity jars,
  checks plugin enable logs, and can optionally drive accepted and banned
  protocol logins through Velocity. JVM tests exercise `/hub` and `/lkjmc send`
  with faked Velocity players and profile-save acknowledgements.
- Installer and live Minecraft smokes are not part of default verification
  because they are slow and require nested Docker or network/server downloads.
- Config reload applies roots and database settings to new daemon operations;
  existing child process launch directories are not rewritten in place.

## Verification status

The meaningful acceptance checks are foundation, pure-core, store, daemon API,
and the local process runtime slice. Process runtime checks require a real
PostgreSQL URL and are skipped by local verification when it is absent.
