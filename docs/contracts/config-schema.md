# Config schema coverage

## Purpose

This contract keeps Rust-owned runtime config fields and JVM runtime validation
from drifting silently.

## Covered fields

The generated or mirrored contract covers `LKJMC_DAEMON_HTTP_URL`,
`LKJMC_DAEMON_HTTP_TOKEN`, `LKJMC_DAEMON_HTTP_TOKEN_FILE`, `LKJMC_INSTANCE_ID`,
`LKJMC_PLATFORM_ROLE`, `LKJMC_DEFAULT_LOCALE`, public host display fields, web
listener settings, runtime adapter kind, and Kubernetes adapter settings.

## Drift rule

Rust remains the canonical owner for product JSON config. The JVM common module
loads the schema artifact or mirror list during tests and validates the subset
that plugins consume. A deterministic script fails if required field names are
present in Rust config docs but absent from the Java contract resource.

## Diagnostics

Validation failures map to typed diagnostics: missing config, unreadable token
file, invalid URL, invalid instance id, invalid locale, dependency unavailable,
auth denied, and schema mismatch. Plugins must fail early rather than register
misleading live actions after fatal runtime config errors.
