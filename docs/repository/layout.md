# Layout

## Purpose

This document defines repository layout and distinguishes authored source from
ignored or generated output.

## Roots

- `docs/`: implementation contracts and state documentation
- `contracts/`: machine-readable cross-language contract data
- `scripts/`: local entry points, static checks, and verification tiers
- `migrations/`: PostgreSQL migrations
- `crates/`: Rust workspace crates for core, store, daemon, CLI, Discord, and
  xtask; daemon root files stay thin while owned domains live below them
- `platforms/jvm/`: Java common, Velocity, and Paper/Folia modules
- `config/`: locale catalogs and safe example defaults; user config remains JSON
- `tests/smoke/`: Java smoke harness sources used by guarded shell checks

## Generated artifact policy

Both `.gitignore` and `.dockerignore` exclude top-level `tmp/`, `target/`,
`data/`, `runtime/`, `logs/`, Gradle state, nested `**/build/` and `**/target/`,
and jars, logs, databases, sockets, and PIDs; only `.gitignore` also excludes
`out/`. Build archives, server worlds,
container volumes, databases, logs, and generated secrets must remain outside
commits. Never print generated secrets.

Ignore policy is a version-control and build-context boundary, not a universal
scanner boundary. `scripts/check-lines.py` has its own recursive skip list. It
skips a directory only when that directory is the first path component, so it
currently does not honor nested Gradle `build/` output. See
[Contract checks](contract-checks.md) for the known line-check defect and
verification meaning.

## Traceability rule

For a repository claim, cite the source file or script that implements it. For a
verification claim, cite the exact command and report whether it passed, failed,
or skipped. A generated file is evidence only when its producing command and
its clean-up or ignore boundary are also named.
