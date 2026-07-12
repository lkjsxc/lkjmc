# Config schema coverage

## Purpose

This contract scopes Rust-owned JSON runtime configuration and each accepted
field's source owner. It does not define a Java daemon-client configuration or
prove a service started.

## Current boundary

Rust parses the daemon example with `LkjmcConfig::from_json_str`; its serde
models reject unknown object fields. `contracts/config/owners.json` is the
selected ownership inventory. It maps each accepted example leaf to the Rust
source that validates or consumes it. The inventory is checked against the
example and real loader.

Local-safe Java plugins consume no daemon URL, token file, instance id, role,
or runtime feature flag. The former JVM config mirror remains withdrawn pending
trusted identity and session attestation.

## Diagnostics

Rust validation emits typed configuration diagnostics without printing token
bytes. A parser test does not prove daemon reachability, web login, or an
external surface. Java plugins do not register a fallback daemon action when
configuration is absent.

## Verification

`scripts/check-contracts.py --probe all-config-fields-owned` checks ownership
records and invokes the existing Rust-parser example check. `check-config-schema.py`
continues to reject a Java daemon-config mirror.
