# Control plane state

## Purpose

This matrix records bounded shipped Rust control-plane capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Command catalog parsing and daemon registration | [registry](../contracts/command-registry.md) | `crates/lkjmc-core/src/command_registry.rs`; `crates/lkjmc-daemon/src/commands/command_registrations.rs` | `crates/lkjmc-daemon/src/tests/command_registry_tests.rs`; `scripts/check-command-docs.py` | none | The generic request and response schemas do not express handler fields or effect semantics. | `F-CLAIM-PROBES`, `A-CONTRACT` |
| PostgreSQL-backed daemon command transport | [daemon](../architecture/runtime/daemon/README.md) | `crates/lkjmc-daemon/src/transport/command.rs`; `crates/lkjmc-daemon/src/commands/mod.rs` | `crates/lkjmc-daemon/src/tests/api_tests.rs` | `LKJMC_CLAIM_SMOKE=1 ./scripts/check-claim-smoke.sh` | A successful transport response does not prove an external runtime effect. | `A-EXECUTION` |
| Selected runtime adapter planning and observation | [adapters](../architecture/runtime/adapters.md) | `crates/lkjmc-core/src/kubernetes.rs`; `crates/lkjmc-core/src/config/runtime_validate.rs` | `crates/lkjmc-core/src/kubernetes_tests.rs` | `LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` | The guarded smoke only observes a listed ID; logs, stop/delete state, and recovery are unproved. | `D-OPS`, `F-SAFE-RUNTIME` |

## Boundary

This matrix does not claim runtime serialization, handler-specific schemas, or
external effects beyond the named proofs.
