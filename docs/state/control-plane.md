# Control plane state

## Purpose

This matrix records bounded shipped Rust control-plane capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Closed command-member validation and daemon registration | [registry](../contracts/command-registry.md) | `contracts/commands/README.json`; `crates/lkjmc-core/src/command_registry.rs`; `crates/lkjmc-daemon/src/commands/command_registrations.rs` | `crates/lkjmc-core/src/command_registry_tests.rs`; `scripts/check-contracts.py` | none | The 137 contracts reject undeclared body members, but response bodies, effects, replay, and deadlines are `not-established` unless separately proved. | `A-EXECUTION` |
| PostgreSQL-backed daemon command transport | [daemon](../architecture/runtime/daemon/README.md) | `crates/lkjmc-daemon/src/transport/command.rs`; `crates/lkjmc-daemon/src/commands/mod.rs` | `crates/lkjmc-daemon/src/tests/api_tests.rs` | `LKJMC_CLAIM_SMOKE=1 ./scripts/check-claim-smoke.sh` | A successful transport response does not prove an external runtime effect. | `A-EXECUTION` |
| Selected runtime adapter planning and observation | [adapters](../architecture/runtime/adapters.md) | `crates/lkjmc-core/src/kubernetes.rs`; `crates/lkjmc-core/src/config/runtime_validate.rs` | `crates/lkjmc-core/src/kubernetes_tests.rs` | `LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` | The guarded smoke only observes a listed ID; logs, stop/delete state, and recovery are unproved. | `D-OPS`, `F-SAFE-RUNTIME` |

## Boundary

This matrix does not claim runtime serialization, typed response bodies, or
external effects beyond the named proofs.
