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

## Playable deterministic gates

`./scripts/check-bootstrap-docs.py` and `./scripts/check-asset-docs.py` are now
part of the default verification script. They check docs and source catalog
alignment only; they do not download assets or start servers.

## PostgreSQL integration

Daemon tests cover claim command dispatch, and store integration tests cover
durable helpers. Process runtime, claim, and jar smokes run when their
environment flags and `LKJMC_STORE_TEST_DATABASE_URL` are set. Compose verify
sets it to the Compose PostgreSQL service.

## Optional live checks

See [smoke-checks.md](smoke-checks.md) for installer, live jar, live Minecraft,
live Paper claim, playable Java, plugin asset, and Bedrock smoke commands that
remain opt-in unless a stable cache strategy makes them deterministic.

## Compose gates

Current verify gate:

```sh
docker compose -f docker-compose.yml -f docker-compose.verify.yml run --rm verify
```

Playable target gate:

```sh
LKJMC_ACCEPT_MINECRAFT_EULA=1 \
  docker compose -f docker-compose.yml -f docker-compose.playable.yml \
  up --build --abort-on-container-exit playable
```

Docker-unavailable environments must report the gate as not run, not passed.
