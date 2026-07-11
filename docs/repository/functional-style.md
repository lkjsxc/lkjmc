# Functional style

## Purpose

This document defines coding style for pure cores and effect adapters.

## Source examples

| Boundary | Pure description | Effect adapter |
| --- | --- | --- |
| bootstrap | `crates/lkjmc-core/src/bootstrap/plan.rs` returns a plan | `crates/lkjmc-core/src/bootstrap/effect.rs` describes effects for a caller to perform |
| runtime | `crates/lkjmc-daemon/src/runtime/adapter.rs` defines observations and capability contract | `crates/lkjmc-daemon/src/runtime/local_adapter.rs` invokes the local runtime |

A plan or decision may select an effect but must not perform filesystem, process,
network, or database work while deciding. An adapter may perform that work and
must return an observation or error that the core can reason about.

## Rust

- Model states with explicit structs and enums.
- Pure planners return decisions and effect descriptions.
- Effect adapters execute process, network, filesystem, and database work.
- Product crates avoid `unwrap`, `expect`, `panic`, `todo`, and `unimplemented`.
- Parse JSON once at boundaries and use typed structs internally.
- Keep state machine tests exhaustive and deterministic.

## Java

- Common module stays platform-neutral and prefers records and sealed interfaces.
- Paper/Folia and Velocity modules are adapters only.
- Menu behavior must use metadata, not display names.
- Scheduler crossing is explicit, and daemon calls are asynchronous.
- Locale keys are stable and tested in English and Japanese.

## Docs

- Put one concept in one owner doc.
- Cite repository-relative source paths for shipped behavioral claims.
- Keep documents actionable: purpose, contract, failure behavior, verification.
- Mark future behavior as future until implementation is real.
