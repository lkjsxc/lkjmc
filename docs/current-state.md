# Current state

## Purpose

This ledger states what is implemented now. If it conflicts with any target
contract, this file wins for current behavior.

## Field incident boundary

The command/menu/auth incident is covered by the opt-in playable smoke. With
EULA acceptance, it proves `/lkjmc`, completion, token-file auth, `/menu`, no
unintended close, `/docs`, shop purchase delivery, and cobblestone exchange.

## Repository and verification

- Documentation topology, line-limit, bootstrap-doc, asset-doc, command-doc,
  permission-doc, and locale catalog checks are implemented.
- `./scripts/verify.sh` runs docs, contract drift checks, Rust
  formatting/lint/tests, daemon/CLI, process runtime, jar registry, installer,
  Minecraft smoke guards, Java tests, and shaded plugin jar assembly.
- Dockerfile, Compose verify scaffolding, a playable Compose wrapper, and
  protocol command/menu smoke harness are implemented.
- Daemon and opt-in smokes cover claim dispatch, live Paper claim behavior, and
  protocol-level break/place protection when prerequisites are set.
- Installer, playable Compose, and live Minecraft smokes opt in because they need
  privileged host changes, Docker, EULA acceptance, or network downloads.

## Rust control plane

- The Cargo workspace contains `lkjmc-core`, `lkjmc-store`, `lkjmc-daemon`,
  `lkjmc-cli`, `lkjmc-discord`, and `lkjmc-xtask`; installer behavior lives in
  scripts.
- `lkjmc-core` has pure models for IDs, instances, jars, players, commands,
  admin role permissions, audit events, reconciliation effects, playable bootstrap planning,
  autosuspend planning, adventure catalogs, achievement definitions, rich economy
  defaults, random teleport policy, temporary adventure state helpers and
  allocation planning, server capabilities, and JSON config validation.
- PostgreSQL migrations create durable tables for core runtime, profiles,
  economy, achievements, shop, admin RBAC/audit, gameplay, moderation, claims,
  commands, outbox, temporary adventures, transfers, and wake-and-join.
- `lkjmc-store` applies migrations and provides typed helpers for the tables
  named in [architecture/data/schema.md](architecture/data/schema.md), including
  instance presence, assets, plugin installations, bootstrap run ledgers,
  temporary instances, adventure sessions, transfer intents, random teleport
  reservations/history, achievement reward claims, point and mail reward
  delivery, rich shop catalog seeding, and wake-and-join queue rows.
- `lkjmc-daemon` serves Unix socket JSON-RPC, token-protected loopback HTTP
  commands, and private authenticated `/web` operator pages. Browser login uses
  the configured HTTP token source, stores in-memory sessions tied to the token
  fingerprint, requires CSRF on cookie-backed form posts, and keeps bearer-safe
  `/web/api/` mutation paths. Documented admin command families pass through
  daemon authorization that accepts local CLI, web, platform permission input,
  or durable admin grants.
- `lkjmc-daemon` serves claim create/delete/list/snapshot/trust/untrust commands
  backed by PostgreSQL and audit events.
- `lkjmc-daemon` serves temporary instance create/start/stop/cleanup/get,
  transfer intent, catalog adventure, and End Expedition compatibility purchase
  commands backed by PostgreSQL, generated world directories, local process
  runtime, verified Folia jars, required `lkjmc` Paper plugin installation,
  readiness probes, retention checks, cleanup worker, point ledger spend,
  startup failure refund, and audit events.
- Daemon command coverage, including startable instance create planning and
  homes and warps list/get/set commands, is cataloged in
  [architecture/runtime/daemon/command-catalog.md](architecture/runtime/daemon/command-catalog.md), including random teleport quote/reserve/complete/refund/history.
- The daemon accepts HTTP bearer token text or `--http-token-file`, rotates the
  configured token file atomically, hot-swaps HTTP auth, verifies old/new token
  behavior in tests, and audits safe fingerprints. Managed JVM token-file auth
  is proven by playable smoke; Java clients reread token files for rotation.
- `status` reports daemon start/uptime, database configuration/connectivity,
  PostgreSQL counts, roots, socket path, HTTP listener state, runtime adapter and
  capabilities, and reconciler state.
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
- Runtime orchestration has `local-process` and `kubernetes` selectable adapters
  after config validation. Local orchestration supports process recovery,
  TERM/KILL, logs, ports, and templates. Kubernetes plans owned manifests,
  applies with `kubectl`, parses typed pod JSON observation, reads bounded logs,
  scales stop, recovers observation from owned pods, and deletes owned objects.
- Jar registry import, PaperMC stable sync, Purpur sync, prune, list, inspect,
  checksum verification, Java 21-compatible default Paper/Folia release
  selection with available 1.21 fallback, and opt-in live PaperMC download smoke
  are implemented.
  Asset server sync wraps server sync, and asset plugin sync/list/inspect handle
  local lkjmc plugin assets, Modrinth ViaVersion/ViaBackwards assets, and
  GeyserMC Geyser/Floodgate proxy assets. Playable bootstrap plans Folia as the
  default hub backend and installs supported third-party plugin downloads on
  hub/proxy.
- The CLI supports doctor/status, config, database, audit, verify, bootstrap,
  network diagnose, jar, instance, claim, admin, shop seeding, kit, vote,
  announcement, player, and moderation families. Bootstrap apply executes real
  effects and fails for missing roots, migrations, jars, plugin builds, secrets,
  starts, readiness timeouts, or required optional-plugin assets.
- `lkjmc-discord` is a real Rust service crate. It validates JSON config and
  token sources without printing secrets, registers the `/lkjmc` slash-command
  tree through Discord REST when configured, verifies signed interaction HTTP
  requests, maps Discord users and roles into daemon principal evidence, defers
  daemon-backed interactions, sends follow-up responses, can perform a daemon
  status check over authenticated loopback HTTP, formats daemon server/report
  lists for Discord follow-ups, and is wired as an opt-in Compose profile and
  guarded smoke script. Link-required commands report that requirement instead of
  faking success.
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
  HTTP daemon config and diagnostics, runtime config schema validation,
  Gson-backed typed daemon JSON transport, shared `/lkjmc` command tree parsing,
  typed parse failures, completion metadata, localization,
  permission constants, metadata-driven menu records with structured payload
  maps, menu reducers, generated-name helpers, pure route-stack navigation,
  shared menu chrome, typed menu diagnostics, docs browser path/wrap/page helpers,
  route-derived docs parent navigation, persisted-locale resolution, action-bar
  formatting/frame builders, dedupe reducer, transfer records, and tests.
- Velocity source registers `/lkjmc` as a Brigadier graph generated from the
  shared JVM command specs, with intended product usage on root and intermediate
  branches, dynamic argument suggestions, shared execution targets, fallback
  product no-permission handling for hidden Brigadier nodes, and daemon-backed
  `instance.list` output. Paper and Velocity keep asynchronous admin grant
  snapshots so fresh durable grants affect `/lkjmc` visibility, completion, and
  admin menu enabled states before execution while daemon authorization remains
  final. The shared tree now includes admin, config, security, economy, and
  adventure command families where daemon-backed execution exists. The current
  playable smoke proves the documented root, status, doctor, server usage,
  server list, and completion paths.
- Paper/Folia registers the commands listed in
  [product/commands/minecraft.md](product/commands/minecraft.md), exposes the
  public plugin identity as `lkjmc`, wires `/lkjmc` execution and tab completion
  to the shared command tree, implements `/exchange` inventory removal with
  daemon commit and refund-on-failure, packages a generated docs bundle and
  exposes `/docs` inventory browsing with Main Menu, Parent Directory chrome,
  and file-page Previous/Next controls adjacent to the content item, uses a
  Folia-aware scheduler bridge, sends heartbeats, opens localized menus,
  applies join-time profiles, records sessions, handles chat, claim, profile,
  achievement reward claims, paid random teleport, portal cancellation, passive
  action-bar snapshots, persisted language caching, and transfer adapter work.
- Source adapters include live-data menus, true empty states, admin-gated rows,
  list-first Admin server detail/confirm routes, kind/template Admin server
  creation flow with a free-form id prompt, generated lower-left home creation,
  one-click party creation, random teleport quote rows, shop balance/category/
  affordability rows, catalog adventure rows, and typed diagnostics. Current playable
  smoke proves disabled rows, daemon actions, and no unintended close on covered
  routes. Root slots `30` and `31` open Documentation and Admin. Hotbar slot `8`
  remains a `NETHER_STAR` token.
- English and Japanese locale catalogs exist in repository config and Java
  resources with matching key sets; persisted language now beats platform locale
  for Paper menu, command, docs, shop errors, and action-bar rendering.

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
- Temporary instance lifecycle covers local Folia create/start/readiness/stop,
  cleanup, Velocity hints, transfer intents, catalog adventure commands, End
  Expedition compatibility, startup refunds, `/endexpedition` solo/party/return,
  catalog menu rows, and generic adventure shop delivery. Wake-and-join has
  durable request, status, cancellation, cleanup, consume, menu wake, and
  Velocity transfer safety paths.
- Live Minecraft, playable Compose, and live Paper claim smoke automation are
  implemented or wired as opt-in paths and remain outside default verification.
  The playable command/menu smoke joins through Velocity and now proves exact
  command, completion, auth, menu data, docs, shop, exchange, and no-close paths.

## Verification status

Default verification is meaningful for docs, pure core, store, daemon API, CLI,
Java common/plugins, local process runtime, and jar registry slices. PostgreSQL
runtime checks run when `LKJMC_STORE_TEST_DATABASE_URL` is set. The opt-in
playable smoke currently proves the Minecraft-facing command, completion, auth,
menu, docs, shop, and exchange incident when Docker, EULA acceptance, and
network downloads are available.
