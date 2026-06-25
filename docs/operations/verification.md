# Verification

## Purpose

This document defines current and target verification gates.

## Current gates

```sh
./scripts/check-lines.py
./scripts/check-docs.py
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-daemon-cli.sh
./scripts/check-process-runtime.sh
./scripts/check-jar-registry.sh
./scripts/check-installer.sh
./scripts/check-minecraft-smoke.sh
./gradlew --no-daemon test shadowJar
./scripts/verify.sh
```

`./scripts/verify.sh` suppresses successful subcommand output and prints:

```text
ok verify
```

## PostgreSQL integration

Store integration tests, the process runtime smoke gate, and the jar registry
smoke gate run when `LKJMC_STORE_TEST_DATABASE_URL` is set. The Compose verify
service sets it to the Compose PostgreSQL service. Runtime and jar smokes reset
that test database through the gated `LKJMC_TEST_RESET_DATABASE=1 lkjmc db
reset-test` helper before creating processes.

## Installer and live jar smoke

`scripts/check-installer.sh` prints `ok installer skipped` by default. Set
`LKJMC_INSTALLER_SMOKE=1` to run the clean Ubuntu container installer smoke.
Set `LKJMC_JAR_LIVE_SMOKE=1` while running `scripts/check-jar-registry.sh` with
PostgreSQL configured to download a live PaperMC stable server jar.
`LKJMC_MINECRAFT_SMOKE=1 ./scripts/check-minecraft-smoke.sh` downloads real
Paper and Velocity server jars, installs the built plugin jars, starts each
server, and checks that the lkjmc plugin enable messages appear. Adding
`LKJMC_MINECRAFT_PLAYER_SMOKE=1` and `LKJMC_STORE_TEST_DATABASE_URL` starts the
real daemon HTTP API and should verify both accepted and banned offline-mode
Velocity logins. JVM tests smoke the player-driven `/hub` and `/lkjmc send`
transfer command paths with faked Velocity players.

## Compose gate

```sh
docker compose -f docker-compose.yml -f docker-compose.verify.yml run --rm verify
```

The compose gate runs the current local verification script inside a copied
repository image with PostgreSQL available, including JVM tests and shaded
plugin jar assembly.
