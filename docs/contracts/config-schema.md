# Config schema coverage

## Purpose

This contract scopes the JVM-consumed subset of Rust-owned runtime config. It
prevents silent field-list drift; it does not define every daemon JSON setting
or prove that a configured service has started.

## Covered fields

The generated or mirrored contract covers `LKJMC_DAEMON_HTTP_URL`,
`LKJMC_DAEMON_HTTP_TOKEN`, `LKJMC_DAEMON_HTTP_TOKEN_FILE`, `LKJMC_INSTANCE_ID`,
`LKJMC_PLATFORM_ROLE`, `LKJMC_DEFAULT_LOCALE`, public host display fields, web
listener settings, runtime adapter kind, and Kubernetes adapter settings.

## Owner evidence and drift rule

Rust field owner evidence is `crates/lkjmc-core/src/config/schema.rs`; the JVM
mirror is `platforms/jvm/common/src/main/resources/lkjmc-config-contract.json`.
`check-config-schema.py` compares those two field sets and requires every Rust
listed field in this document. It is deterministic contract proof, not a full
JSON-schema validator or runtime connectivity proof.

Rust remains canonical for product JSON configuration. The JVM common module
validates only the subset plugins consume. `config/defaults/daemon.json.example`
must stay in the current camelCase `LkjmcConfig` shape;
`check-config-examples.py` rejects obsolete `paths`, `http`, and `database.url`
example drift before the Rust parser test runs.

## Diagnostics

Validation failures map to typed diagnostics: missing config, unreadable token
file, invalid URL, invalid instance id, invalid locale, dependency unavailable,
auth denied, and schema mismatch. Plugins must fail early rather than register misleading live actions after
fatal runtime config errors. A passing parser or mirror test does not prove
plugin startup, daemon reachability, token acceptance, or an external surface;
those need implementation, Compose, or live evidence at their respective level.
