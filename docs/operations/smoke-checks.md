# Smoke checks

## Purpose

This document defines optional checks with real, bounded external behavior.

## Status

implemented

## Supported opt-in checks

- `LKJMC_INSTALLER_SMOKE=1 ./scripts/check-installer.sh` runs a clean Ubuntu
  installer check.
- `LKJMC_JAR_LIVE_SMOKE=1 ./scripts/check-jar-registry.sh` downloads a live
  PaperMC server jar when PostgreSQL is configured.
- `LKJMC_CLAIM_SMOKE=1 ./scripts/check-claim-smoke.sh` exercises daemon and CLI
  claim operations with a PostgreSQL test URL.
- `LKJMC_PLUGIN_LIVE_SMOKE=1 ./scripts/check-plugin-assets.sh` verifies live
  third-party asset download and hash checks.
- `LKJMC_BEDROCK_SMOKE=1 ./scripts/check-bedrock-smoke.sh`,
  `LKJMC_WEB_SMOKE=1 ./scripts/check-web-smoke.sh`, and
  `LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` require their
  documented endpoints, credentials, and disposable environment.

## Withdrawn Java smoke paths

`check-minecraft-smoke.sh`, `check-minecraft-claim-smoke.sh`, and
`check-playable-smoke.sh` are retained only as explicit blocked diagnostics. If
their `LKJMC_*_SMOKE=1` guard is set, they fail and cannot report a Java daemon
adapter success. The former command-menu and claim-protocol harnesses are not
shipped. No local-safe Java live smoke is claimed until it has a real source,
assertion, and bounded owner contract.

## Live evidence

Run `./scripts/verify-live.sh` only after setting a supported guard and every
external prerequisite for the intended lane. It reports unguarded supported
lanes as skipped. A guarded script may fail when a prerequisite is missing.
Preserve the command, environment names without secret values, final result, and
redacted evidence.

## Rule

A skip is not a pass. A blocked Java adapter smoke is not a skip or a pass. A
passing deterministic or Compose gate is not live proof.
