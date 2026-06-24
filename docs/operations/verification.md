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
./gradlew --no-daemon test
./scripts/verify.sh
```

`./scripts/verify.sh` suppresses successful subcommand output and prints:

```text
ok verify
```

## Compose gate

```sh
docker compose -f docker-compose.yml -f docker-compose.verify.yml run --rm verify
```

The compose gate runs the current local verification script inside a copied
repository image. Database migrations and daemon checks are not part of it yet.
