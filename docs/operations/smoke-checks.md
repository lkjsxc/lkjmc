# Smoke checks

## Purpose

Define bounded transport and authentication checks without claiming an external
command effect.

## Status

implemented

## Available checks

- `./scripts/check-daemon-cli.sh` proves status succeeds, doctor is denied, and
  optional `LKJMC_ASSERT_SHUTDOWN=1` proves the daemon process exits on TERM.
- `./scripts/check-process-runtime.sh` proves an instance start is denied before
  a process starts.
- `./scripts/check-jar-registry.sh` proves jar import is denied before a
  filesystem write.
- `LKJMC_CLAIM_SMOKE=1 ./scripts/check-claim-smoke.sh` proves claim deletion is
  denied before database work.
- `LKJMC_WEB_SMOKE=1 ./scripts/check-web-smoke.sh` proves web authentication and
  CSRF boundaries; it does not prove a command mutation.

## External lanes

Installer, asset download, Minecraft, Bedrock, Kubernetes
(`check-kubernetes-smoke.sh`), browser, remote-world, and live-player lanes
remain unavailable as command-effect proof.
Their guards are not support claims and may only return a skip, block, or
failure until an owner closes the external completion boundary.

## Rule

A passing deterministic, process, or Compose check is not live external proof.
A non-success `command.effect_denied` is the required result for an unproved
external command, never a fallback success or synthetic completion.
