# Smoke checks

## Purpose

This document defines optional smoke checks that exercise slow or external
runtime behavior.

## Opt-in checks

- `LKJMC_INSTALLER_SMOKE=1 ./scripts/check-installer.sh` runs a clean Ubuntu
  installer smoke.
- `LKJMC_JAR_LIVE_SMOKE=1 ./scripts/check-jar-registry.sh` downloads a live
  PaperMC server jar when PostgreSQL is configured.
- `LKJMC_CLAIM_SMOKE=1 ./scripts/check-claim-smoke.sh` starts the daemon with a
  PostgreSQL test URL, creates a claim, trusts a player, verifies snapshot and
  CLI list output, deletes the claim, and verifies the snapshot is empty.
- `LKJMC_MINECRAFT_SMOKE=1 ./scripts/check-minecraft-smoke.sh` downloads and
  starts standalone Paper and Velocity jars with built plugin jars.
- Adding `LKJMC_MINECRAFT_PLAYER_SMOKE=1` and a PostgreSQL test URL drives
  accepted and banned offline-mode Velocity login checks.
- `LKJMC_MINECRAFT_CLAIM_SMOKE=1 ./scripts/check-minecraft-claim-smoke.sh`
  downloads and starts a real Paper jar, starts the daemon HTTP API, and waits
  for the Paper plugin to create, trust, snapshot, decide, and delete a claim.
- Add `LKJMC_MINECRAFT_CLAIM_PROTOCOL_SMOKE=1` to the live Paper claim smoke to
  run a protocol client that joins as real players, issues `/claim`, and sends
  break/place packets against the claimed chunk.

## Rule

Skipped live checks are reported as skipped and must never be described as
passed.
