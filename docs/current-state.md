# Current state

## Purpose

This ledger states what is implemented now. If it conflicts with any target
contract, this file wins for current behavior.

## Implemented

- Repository documentation skeleton is implemented.
- Line-limit checks are implemented in `scripts/check-lines.py`.
- Documentation topology checks are implemented in `scripts/check-docs.py`.
- Cargo workspace scaffolding is implemented for five Rust crates.
- Minimal Rust crate tests compile and run.
- Gradle multiproject scaffolding is implemented for Java common, Velocity, and
  Paper/Folia modules.
- Minimal Java modules compile through `./gradlew --no-daemon test`.
- Dockerfile and Compose verify scaffolding are implemented.
- `scripts/verify.sh` runs docs, Rust, and Java foundation checks.

## Not implemented

- Runtime daemon behavior is not implemented yet.
- CLI behavior is not implemented yet.
- PostgreSQL schema and migrations are not implemented yet.
- Velocity plugin behavior is not implemented yet.
- Paper/Folia plugin behavior is not implemented yet.
- Installer is not implemented yet; `scripts/install.sh` exits with failure.
- Player synchronization is not implemented yet.

## Verification status

The meaningful acceptance checks are foundation checks until product behavior is
added.
