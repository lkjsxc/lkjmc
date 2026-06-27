# Functional style

## Purpose

This document defines coding style for pure cores and effect adapters.

## Rust

- Model states with explicit structs and enums.
- Pure planners return decisions and effect descriptions.
- Effect adapters execute process, network, filesystem, and database work.
- Product crates avoid `unwrap`, `expect`, `panic`, `todo`, and
  `unimplemented`.
- Parse JSON once at boundaries and use typed structs internally.
- Keep state machine tests exhaustive and deterministic.

## Java

- Common module stays platform-neutral and prefers records and sealed
  interfaces.
- Paper/Folia and Velocity modules are adapters only.
- Menu behavior must use metadata, not display names.
- Scheduler crossing is explicit, and daemon calls are asynchronous.
- Locale keys are stable and tested in English and Japanese.

## Docs

- Put one concept in one owner doc.
- Keep documents actionable: purpose, contract, failure behavior, verification.
- Mark future behavior as future until implementation is real.
