# Build system

## Purpose

This document defines repository build ownership.

## Rust

The Rust workspace owns core models, store helpers, daemon adapters, CLI parsing,
and local runtime orchestration. Default gates run `cargo fmt`, clippy, and
workspace tests.

## Java

Gradle builds Java 21 platform plugins and common JVM contracts. `shadowJar`
outputs are the source for target `lkjmc` plugin assets and must remain real
artifacts, not placeholders.

## Docker

Compose defines PostgreSQL plus `verify`, `playable`, and `discord` profiles in
one file. The Dockerfile has toolchain, Rust dependency, Gradle dependency,
verify, and playable stages so dependency layers can be reused. The playable
target adds a service where the daemon owns child Velocity and Paper processes.

## Release naming

Docs and generated names must not use artificial product release labels. Use
`dev`, commit identifiers, or content hashes when a machine field needs a value.
