# Functional style

## Purpose

This document defines coding style for pure cores and adapters.

## Rust

- Model states with explicit structs and enums.
- Pure functions return decisions and effect descriptions.
- Product crates avoid `unwrap`, `expect`, `panic`, `todo`, and
  `unimplemented`.
- Use precise error enums and newtypes for identifiers and units.

## Java

- Prefer records and sealed interfaces for data and decisions.
- Keep platform API imports at adapter edges.
- Use immutable collections for returned data.
- Use futures or executors for I/O.
- Command handlers parse, authorize, and delegate.
