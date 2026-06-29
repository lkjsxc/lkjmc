# Current state

## Purpose

This ledger states what is implemented now. If it conflicts with any target
contract, this file wins for current behavior.

## Field incident boundary

The command/menu/auth field incident is now covered by an opt-in playable smoke.
With EULA acceptance, the smoke joins the managed network through Velocity,
verifies daemon token-file auth with a mixed-case token, `/lkjmc` output,
completion, `/menu`, server-list data, travel/economy/social empty states,
party, achievements, language selection, settings daemon actions, and a
player-profile menu through protocol packets. Keep this gate green before
claiming future changes preserve it.

## Repository and verification

- Documentation topology, line-limit, bootstrap-doc, asset-doc, command-doc,
  permission-doc, and locale catalog checks are implemented.
- `./scripts/verify.sh` runs docs, contract drift checks, Rust
  formatting/lint/tests, daemon/CLI, process runtime, jar registry, installer,
  Minecraft smoke guards, Java tests, and shaded plugin jar assembly.
- Dockerfile, Compose verify scaffolding, a playable Compose service wrapper,
  and a protocol command/menu smoke harness are implemented.
- Daemon tests cover claim create/trust/list/snapshot/delete dispatch.
- Opt-in claim smoke coverage starts the daemon, creates/trusts/lists/deletes a
  claim through PostgreSQL and CLI surfaces, and verifies snapshots.
- Opt-in live Paper claim smoke starts a real Paper jar and daemon HTTP API,
  then the Paper plugin creates, trusts, snapshots, decides, and deletes a claim.
- An additional opt-in claim protocol smoke joins Paper as real offline-mode
  players, issues `/claim`, and sends break/place packets against the claim.
- Installer, playable Compose, and live Minecraft smoke checks are available but
  opt in because they need privileged host changes, Docker, or network/server
  downloads. The playable command/menu smoke closes the reported `/lkjmc`,
  completion, auth, and menu incident when run with EULA acceptance.

## Rust control plane

- The Cargo workspace contains `lkjmc-core`, `lkjmc-store`, `lkjmc-daemon`,
  `lkjmc-cli`, and `lkjmc-installer` slices.
- `lkjmc-core` has pure models for IDs, instances, jars, players, commands,
  audit events, reconciliation effects, playable bootstrap planning,
  autosuspend planning, temporary adventure state helpers and allocation
  planning, server implementation capabilities, and JSON config validation.
- PostgreSQL migrations create core, instance, presence, jar, generic asset,
  plugin installation, bootstrap run, player profile, settings, sessions,
  points, homes, warps, parties, achievements, shop, kits, votes, teleports,
  mail, reports, warnings, notes, moderation, daily rewards, announcements,
  chunk claims, commands, audit, outbox, temporary instance, adventure session,
  temporary transfer, and wake-and-join queue tables.
- `lkjmc-store` applies migrations and provides typed helpers for the tables
  named in [architecture/data/schema.md](architecture/data/schema.md), including
  instance presence, assets, plugin installations, bootstrap run ledgers,
  temporary instances, adventure sessions, transfer intents, and wake-and-join
  queue rows.
- `lkjmc-daemon` serves Unix socket JSON-RPC and a token-protected loopback HTTP
  command endpoint for plugins.
- `lkjmc-daemon` serves claim create/delete/list/snapshot/trust/untrust commands
  backed by PostgreSQL and audit events.
- `lkjmc-daemon` serves temporary instance create/start/stop/cleanup/get,
  transfer intent, and End Expedition purchase commands backed by PostgreSQL,
  generated world directories, local process runtime, verified Folia jars,
  required `lkjmc` Paper plugin installation, readiness probes, retention
  checks, cleanup worker, point ledger spend, startup failure refund, and audit
  events.
- Daemon command coverage, including homes and warps list/get/set commands, is
  cataloged in
  [architecture/runtime/daemon/command-catalog.md](architecture/runtime/daemon/command-catalog.md).
- The daemon accepts HTTP bearer token text or `--http-token-file`, avoiding
  command-line secrets for managed installs. HTTP auth now preserves credential
  bytes while matching the header name and `Bearer` scheme case-insensitively,
  with tests for mixed-case tokens and token-file newline trim. Managed JVM
  token-file auth is proven by the playable command/menu smoke.
- `status` reports daemon start/uptime, database configuration/connectivity,
  PostgreSQL instance/session/jar/presence counts when available, roots, socket
  path, HTTP listener state, and reconciler state.
- `bootstrap.status` reports instance state, installed plugin state, current
  plan outcome, diagnostics, planned effects, public connection text, and next
  connection steps.
- `doctor` checks config-file intent, root path syntax, socket parent usability,
  HTTP mode, and database connectivity when configured without printing secrets.
- The daemon loads JSON config, can reload roots and database settings, starts a
  periodic reconciler when a database URL is configured, stores heartbeat
  presence with player counts, autosuspends eligible empty backends to desired
  state `suspended`, and recovers stored local process observations after daemon
  restart.
- Local instance orchestration supports create/list/start/stop/restart/delete,
  active-session delete guardrails, bounded logs, explicit launch commands,
  verified jar assets, generated `java -jar` launches, port reservation, and
  template-backed render before launch. The renderer now writes complete-enough
  Velocity defaults, Paper Velocity proxy config, `spigot.yml`, plugin
  directories, and EULA files from explicit config. Paper, Folia, and Purpur
  render through the Paper-compatible server template path.
- Jar registry import, PaperMC stable sync, Purpur sync, prune, list, inspect,
  checksum verification, Java 21-compatible default Paper/Folia release
  selection with available 1.21 fallback, and opt-in live PaperMC download smoke
  are implemented.
  Asset server sync wraps server sync, and asset plugin sync/list/inspect handle
  local lkjmc plugin assets, Modrinth ViaVersion/ViaBackwards assets, and
  GeyserMC Geyser/Floodgate proxy assets. Playable bootstrap plans Folia as the
  default hub backend and installs supported third-party plugin downloads on
  hub/proxy.
- The CLI supports doctor, human and JSON status, config check/reload,
  database migration/status/reset guard, audit tail, verify, bootstrap
  plan/apply/status/doctor, network diagnose, jar, instance with presence-aware
  list output, claim list/delete, shop, kit, vote, announcement, player, and
  moderation families. Bootstrap
  apply executes real effects and fails instead of reporting success for missing
  roots, migrations, jars, plugin builds, secrets, starts, or readiness
  timeouts. Bootstrap effect apply and step recording are exhaustive over the
  effect enum, and enabled optional plugins block instead of auto-withdrawing
  when required assets, dependencies, ports, or safety checks fail.
- Java entry config separates Velocity bind host, TCP port, public hosts, and a
  preferred public host. Bootstrap plan/apply/status/doctor derive defaults from
  loaded config, including runtime memory, port range, daemon HTTP token path,
  and forwarding secret path. CLI overrides merge individual fields, installer
  and playable Compose can write a public host, and status/apply next text
  renders the effective public socket instead of hardcoding loopback.
- `lkjmc network diagnose HOST` resolves A and AAAA through the system resolver,
  queries SRV through DNS, checks TCP and Java status ping, supports direct-IP
  comparison, and emits structured findings and next actions.
- Generated Velocity config binds to the configured Java bind socket and renders
  `forced-hosts` entries for configured public hosts without denying direct-IP
  entry.

## Java and Minecraft adapters

- Java common implements daemon records/client foundation, token-file aware
  HTTP daemon config and diagnostics, Gson-backed typed daemon JSON transport,
  shared `/lkjmc` command tree parsing, typed parse failures, completion
  metadata, localization,
  permission constants, metadata-driven menu records, menu reducers, pure
  route-stack navigation state, shared menu chrome, themed standard menus,
  typed menu diagnostics, transfer records, and tests.
- Velocity registers `/lkjmc` as a Brigadier graph generated from the shared
  JVM command specs, with product usage on root and intermediate branches,
  dynamic argument suggestions, and shared execution targets. It also registers
  `/hub`, server lifecycle commands, `/lkjmc send`, temporary send, wake send,
  reload, restart warning, MOTD, dynamic localhost server registration from
  daemon registration hints, periodic registration refresh and unregister,
  profile-safe transfer coordination, ban login checks, and tab header/footer.
- Paper/Folia registers the commands listed in
  [product/commands/minecraft.md](product/commands/minecraft.md), exposes the
  public plugin identity as `lkjmc`, wires `/lkjmc` execution and tab completion
  to the shared command tree, uses a Folia-aware scheduler bridge, sends
  heartbeats, opens localized menus, applies join-time profiles, records
  sessions, handles chat, claim, profile, and transfer adapter work, and cancels
  scheduled work on disable.
- Source adapters include dynamic menu paths for live daemon data, true empty
  states, and typed diagnostics for missing daemon config, token problems,
  HTTP/auth failure, command failure, database failure, schema mismatch, and
  permission denial. Playable smoke proves ordinary disabled rows and daemon
  actions stay open on covered server, profile, claims, homes, warps, shop,
  kits, votes, daily, mail, reports, party, achievements, language, and settings
  routes except explicit close or manual close. The slot `8` hotbar token
  material is `NETHER_STAR` and retains its marker.
- English and Japanese locale catalogs exist in repository config and Java
  resources with matching key sets, including menu disabled and settings action
  reasons.

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
- Temporary instance daemon lifecycle exists for local Folia create, start,
  readiness, stop, explicit cleanup, scheduled cleanup, Velocity registration
  hints, daemon-validated Velocity transfer intents, End Expedition daemon
  purchase with startup failure refund, live `/endexpedition` solo/party
  transfer, transfer intents, return-to-hub command, automatic pre-expiry return,
  and confirmation menu buttons. Wake-and-join has a durable daemon queue and
  Velocity admin wake-send path for suspended backends.
- Live Minecraft, playable Compose, and live Paper claim smoke automation are
  implemented or wired as opt-in paths and remain outside default verification.
  The playable command/menu smoke joins through Velocity and proves the reported
  command, completion, auth, menu data, menu empty-state, party, achievements,
  language-selection, and settings-action paths with a managed proxy and hub.

## Verification status

Default verification is meaningful for docs, pure core, store, daemon API, CLI,
Java common/plugins, local process runtime, and jar registry slices. PostgreSQL
runtime checks run when `LKJMC_STORE_TEST_DATABASE_URL` is set. The opt-in
playable smoke proves the Minecraft-facing command, completion, auth, and menu
incident when Docker, EULA acceptance, and network downloads are available.
