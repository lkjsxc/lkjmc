# Smoke checks

## Purpose

This document defines optional smoke checks that exercise slow or external
runtime behavior.

## Current opt-in checks

- `LKJMC_INSTALLER_SMOKE=1 ./scripts/check-installer.sh` runs a clean Ubuntu
  installer smoke.
- `LKJMC_JAR_LIVE_SMOKE=1 ./scripts/check-jar-registry.sh` downloads a live
  PaperMC server jar when PostgreSQL is configured.
- `LKJMC_CLAIM_SMOKE=1 ./scripts/check-claim-smoke.sh` starts the daemon with a
  PostgreSQL test URL and verifies claim create, trust, list, snapshot, and
  delete through daemon and CLI surfaces.
- `LKJMC_MINECRAFT_SMOKE=1 ./scripts/check-minecraft-smoke.sh` downloads and
  starts standalone Paper and Velocity jars with built plugin jars.
- Adding `LKJMC_MINECRAFT_PLAYER_SMOKE=1` and a PostgreSQL test URL drives
  accepted and banned offline-mode Velocity login checks.
- `LKJMC_MINECRAFT_CLAIM_SMOKE=1 ./scripts/check-minecraft-claim-smoke.sh`
  starts real Paper with daemon HTTP and verifies claim plugin behavior.
- Add `LKJMC_MINECRAFT_CLAIM_PROTOCOL_SMOKE=1` to join as real players and send
  break/place packets against the claimed chunk.

## Playable smokes

- `LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 \
  ./scripts/check-playable-smoke.sh` runs the playable Compose path, keeps the
  managed proxy and hub alive, and uses a protocol-level offline smoke player to
  assert mixed-case daemon token-file auth, `/lkjmc` output, `/lkjmc`
  suggestions, `/menu`, server-list data, travel/economy/social empty states,
  party, achievements, language selection, settings daemon actions, and one
  daemon-backed player menu. The script sets `LKJMC_PLAYABLE_ONLINE_MODE=false`
  and grants only `LKJMC_SMOKE_ADMIN_PLAYERS` the proxy/backend admin permission
  bypass.
- `LKJMC_PLUGIN_LIVE_SMOKE=1 ./scripts/check-plugin-assets.sh` verifies live
  third-party plugin download and hash checks through daemon asset commands.
- `LKJMC_BEDROCK_SMOKE=1 ./scripts/check-bedrock-smoke.sh` verifies the optional
  UDP Geyser listener when Bedrock support is enabled.

## Rule

Skipped live checks are reported as skipped and must never be described as
passed.
