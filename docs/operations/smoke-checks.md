# Smoke checks

## Purpose

This document defines optional smoke checks that exercise slow or external
runtime behavior.


## Status

implemented

## Current opt-in checks

- `LKJMC_INSTALLER_SMOKE=1 ./scripts/check-installer.sh` runs a clean Ubuntu
  installer smoke.
- `LKJMC_JAR_LIVE_SMOKE=1 ./scripts/check-jar-registry.sh` downloads a live
  PaperMC server jar when PostgreSQL is configured.
- `LKJMC_CLAIM_SMOKE=1 ./scripts/check-claim-smoke.sh` starts the daemon with a
  PostgreSQL test URL and verifies claim create, trust, list, snapshot, and
  delete through daemon and CLI surfaces. Every request needs one nonempty JSON
  response; transport closure, invalid JSON, or daemon exit prints a bounded
  diagnostic and fails the smoke.
- `LKJMC_MINECRAFT_SMOKE=1 ./scripts/check-minecraft-smoke.sh` downloads and
  starts standalone Paper and Velocity jars with built plugin jars.
- Adding `LKJMC_MINECRAFT_PLAYER_SMOKE=1` and a PostgreSQL test URL drives
  accepted and banned offline-mode Velocity login checks.
- Java daemon claim and protocol smokes are withdrawn with the daemon-capable
  Paper adapter; they must not be re-enabled without trusted identity/session
  attestation.

## Playable smokes

- `LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 \
  ./scripts/check-playable-smoke.sh` runs the playable Compose path, keeps the
  managed proxy and hub alive, and uses a protocol-level offline smoke player to
  assert only local-safe Paper `/menu` and `/docs`, hotbar/docs UI, and Velocity
  MOTD/tab-list behavior. It must not assert `/lkjmc`, Java token-file auth,
  daemon-backed menus, grant snapshots, or daemon mutations. Java daemon
  adapters are withdrawn pending trusted identity/session attestation.
- `LKJMC_PLUGIN_LIVE_SMOKE=1 ./scripts/check-plugin-assets.sh` verifies live
  third-party plugin download and hash checks through daemon asset commands.
- `LKJMC_BEDROCK_SMOKE=1 ./scripts/check-bedrock-smoke.sh` verifies the optional
  UDP Geyser listener when Bedrock support is enabled.
- `LKJMC_WEB_SMOKE=1 ./scripts/check-web-smoke.sh` verifies private web bind,
  authentication, status rendering, one daemon-backed mutation, and audit.
- `LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` creates and
  starts a test instance, then only checks that `instance list` contains its ID.
  It attempts logs while ignoring failure and does not assert log content or the
  post-stop/delete cluster state. It also needs `LKJMC_KUBERNETES_CONFIG`,
  `LKJMC_KUBERNETES_DATABASE_URL`, `kubectl`, credentials, and a disposable
  namespace; recovery is not exercised by this smoke.
- Playable smoke may opt into token rotation, public wake-and-join, docs Parent
  Directory/Main Menu navigation, and adventure menu purchases once those flags
  are documented by the scripts.
- `LKJMC_DISCORD_SMOKE=1 ./scripts/check-discord-smoke.sh` verifies redacted
  config loading. With real bot credentials, application id, and guild it can
  remove prior slash-command registrations; it does not prove an action surface.

## Live evidence

Run `./scripts/verify-live.sh` only after setting the guard and every external
prerequisite for the intended lane. It reports each unguarded lane as skipped;
a guarded script may still fail when its own required config, tools, credentials,
or EULA acceptance is missing. Preserve the command, environment names without
secret values, final result, and redacted logs or external observation as proof.

## Rule

Skipped live checks are reported as skipped and must never be described as
passed. A passing deterministic or Compose gate is not live proof.
