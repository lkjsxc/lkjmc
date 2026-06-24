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
./gradlew --no-daemon test
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

## Compose gate

```sh
docker compose -f docker-compose.yml -f docker-compose.verify.yml run --rm verify
```

The compose gate runs the current local verification script inside a copied
repository image with PostgreSQL available. Plugin checks are not part of it
yet.
