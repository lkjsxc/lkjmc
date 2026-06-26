# Verification

## Purpose

This document defines current and target verification gates.

## Current gates

```sh
./scripts/check-lines.py
./scripts/check-docs.py
./scripts/check-command-docs.py
./scripts/check-permissions.py
./scripts/check-locales.py
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

Store integration tests, including `crates/lkjmc-store/tests/claims.rs`, the
process runtime smoke gate, and the jar registry smoke gate run when
`LKJMC_STORE_TEST_DATABASE_URL` is set. The Compose verify service sets it to
the Compose PostgreSQL service. Runtime and jar smokes reset that test database
through the gated `LKJMC_TEST_RESET_DATABASE=1 lkjmc db reset-test` helper
before creating processes.

## Contract drift checks

Command, permission, and locale drift checks are deterministic repository checks.
They compare source-owned command registrations, permission constants, plugin
metadata, and JSON catalog keys with the documentation contracts.

## Optional live checks

See [smoke-checks.md](smoke-checks.md) for installer, live jar, and live
Minecraft smoke commands that are intentionally outside the default fast path.

## Compose gate

```sh
docker compose -f docker-compose.yml -f docker-compose.verify.yml run --rm verify
```

The compose gate runs the current local verification script inside a copied
repository image with PostgreSQL available, including JVM tests and shaded
plugin jar assembly.
