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
service sets it to the Compose PostgreSQL service.

## Installer smoke

`scripts/check-installer.sh` prints `ok installer skipped` by default. Set
`LKJMC_INSTALLER_SMOKE=1` to run the clean Ubuntu container installer smoke.

## Compose gate

```sh
docker compose -f docker-compose.yml -f docker-compose.verify.yml run --rm verify
```

The compose gate runs the current local verification script inside a copied
repository image with PostgreSQL available, including JVM tests and shaded
plugin jar assembly.
