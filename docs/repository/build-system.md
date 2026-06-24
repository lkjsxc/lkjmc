# Build system

## Purpose

This document defines the current build foundation contract.

## Rust

The root Cargo workspace contains these crates:

- `lkjmc-core`
- `lkjmc-store`
- `lkjmc-daemon`
- `lkjmc-cli`
- `lkjmc-xtask`

Workspace lints deny unsafe code and common panic-path shortcuts.

## Java

The root Gradle build contains these projects:

- `platforms:jvm:common`
- `platforms:jvm:velocity`
- `platforms:jvm:paper`

Java modules compile with Java 21. Velocity and Paper modules depend on common.

## Docker

Compose defines PostgreSQL, builder, integration, smoke, and verify services.
The verify image copies the repository into the image and runs local checks from
inside the copy.
