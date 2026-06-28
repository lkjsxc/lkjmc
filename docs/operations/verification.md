# Verification

## Purpose

This document defines current and target verification gates.

## Current gates

```sh
./scripts/check-lines.py
./scripts/check-docs.py
./scripts/check-bootstrap-docs.py
./scripts/check-asset-docs.py
./scripts/check-command-docs.py
./scripts/check-permissions.py
./scripts/check-locales.py
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-daemon-cli.sh
./scripts/check-process-runtime.sh
./scripts/check-jar-registry.sh
./scripts/check-claim-smoke.sh
./scripts/check-installer.sh
./scripts/check-minecraft-smoke.sh
./scripts/check-minecraft-claim-smoke.sh
./scripts/check-playable-smoke.sh
./scripts/check-plugin-assets.sh
./scripts/check-bedrock-smoke.sh
./gradlew --no-daemon test shadowJar
./scripts/verify.sh
```

`./scripts/verify.sh` suppresses successful subcommand output and prints:

```text
ok verify
```

## Command and menu gates

Command work adds shared parser, permission, execution-target, and completion
unit tests for `/lkjmc status`, `doctor`, server lifecycle, proxy transfer,
restart warning, and destructive `confirm` syntax. Menu work adds typed daemon
diagnostic, reducer, metadata codec, close-effect isolation, token policy,
locale completeness, and adapter tests. Live gameplay checks are still needed
before claiming end-to-end player-facing success.

## Autosuspend gates

Autosuspend work adds planner tests, presence store tests when PostgreSQL is
configured, daemon heartbeat tests, and a smoke proving a suspended backend is
not immediately restarted.

## Compose gates

Current verify gate:

```sh
docker compose -f docker-compose.yml -f docker-compose.verify.yml run --rm verify
```

Playable target gate:

```sh
LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 \
  ./scripts/check-playable-smoke.sh
```

The playable smoke owns generated Compose volumes for this project and removes
them before and after the run so stale instance directories cannot mask a
blocked bootstrap.

Docker-unavailable environments must report the gate as not run, not passed.
