# Config schema coverage

## Purpose

This contract scopes Rust-owned JSON runtime configuration. It does not define a
Java daemon-client configuration or prove a service started.

## Current boundary

Rust owns daemon HTTP, instance, runtime, web, and Kubernetes configuration.
Local-safe Java plugins consume no daemon URL, token file, instance id, role, or
runtime feature flag. The former JVM config mirror is withdrawn pending trusted
identity/session attestation.

## Owner evidence

Rust field owner evidence is `crates/lkjmc-core/src/config/schema.rs`.
`config/defaults/daemon.json.example` stays in the current camelCase
`LkjmcConfig` shape; `check-config-examples.py` rejects obsolete examples before
the Rust parser test runs.

## Diagnostics

Rust validation emits typed configuration diagnostics without printing token
bytes. A passing parser test does not prove daemon reachability, web login, or an
external surface. Java plugins do not register a fallback daemon action when
configuration is absent.
