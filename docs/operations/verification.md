# Verification

## Purpose

This document defines verification tiers and guarded smokes.


## Status

implemented

## Tiers

| Tier | Command | Scope | Final line |
| --- | --- | --- | --- |
| Fast | `./scripts/verify-fast.sh` | docs, contract checks, fmt, clippy, Rust tests without external services | `ok verify-fast skips=...` |
| Full | `./scripts/verify-full.sh` | fast scope plus DB-backed tests when configured, daemon/process/jar/claim checks, installer, plugin/web checks, Gradle test and `shadowJar` | `ok verify-full skips=live-smokes` |
| Default | `./scripts/verify.sh` | wrapper for the full tier used by agent prompts and local handoff checks | `ok verify-full skips=live-smokes` |
| Live | `./scripts/verify-live.sh` | opt-in smokes that need external credentials, EULA, Docker networking, or cluster access | `ok verify-live ran=... skipped=...` |

Compose full verification uses the consolidated profile:

```sh
docker compose --profile verify run --rm verify
```

## Opt-in smoke guards

| Smoke | Guard |
| --- | --- |
| Minecraft protocol smoke | `LKJMC_MINECRAFT_SMOKE=1` |
| Minecraft claim protocol smoke | `LKJMC_MINECRAFT_CLAIM_SMOKE=1` |
| Playable Velocity/Paper network | `LKJMC_PLAYABLE_SMOKE=1` and `LKJMC_ACCEPT_MINECRAFT_EULA=1` |
| Bedrock/Geyser smoke | `LKJMC_BEDROCK_SMOKE=1` |
| Discord live smoke | `LKJMC_DISCORD_SMOKE=1` |
| Kubernetes live smoke | `LKJMC_KUBERNETES_SMOKE=1` |

Guarded smokes print skipped when prerequisites are absent. Skipped checks must
not be reported as passed.

## Store and CLI gates

Store integration tests create per-test PostgreSQL schemas named
`lkjmc_test_<random>` and run migrations inside each schema so parallel test
threads do not share tables. CLI parsing has a Rust unit suite covering global
flags, command-family parsing, and usage failures.

## Playable gate

```sh
LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 \
  ./scripts/check-playable-smoke.sh
```

The playable target joins through Velocity and asserts `/lkjmc status`,
`/lkjmc doctor`, `/lkjmc server`, `/lkjmc server list`, completions, `/menu`,
docs navigation, server-list data, one daemon-backed player menu, token-file
daemon auth, and absence of parser or secret leaks. The smoke owns generated
Compose volumes and removes them before and after the run.
