# Control plane state

## Purpose

This matrix records bounded shipped Rust control-plane capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Closed command-member validation and fail-closed effect classes | [registry](../contracts/command-registry.md) | `contracts/commands/README.json`; `crates/lkjmc-core/src/command_registry.rs`; `crates/lkjmc-daemon/src/command_lifecycle.rs` | `crates/lkjmc-daemon/src/command_lifecycle.rs`; `scripts/check-contracts.py`; `scripts/check-command-lifecycle.py` (effect-classes-enforced probe) | none | 137 contracts classify as 1 local observation, 2 PostgreSQL reads, 2 desired-state writes, 1 restart requirement, or 131 denials. | `B-E` |
| Bounded daemon request admission and desired-state outcomes | [lifecycle](../architecture/runtime/daemon/command-lifecycle.md) | `crates/lkjmc-daemon/src/app/`; `crates/lkjmc-daemon/src/transport/`; `crates/lkjmc-daemon/src/dispatch.rs`; `crates/lkjmc-store/src/command.rs`; `crates/lkjmc-store/src/pool.rs` | `scripts/check-command-lifecycle.py`; `crates/lkjmc-daemon/src/app/admission/tests.rs`; `crates/lkjmc-daemon/src/tests/deadline_route_tests.rs`; `crates/lkjmc-daemon/src/tests/command_lifecycle_tests.rs` | Compose verify runs required real-PostgreSQL timeout and replay probes. | Eight shared leases cover auth, denial audit, commands, and web paths. Desired-state mutations retain request correlation and finish in a queryable succeeded, failed, or cancelled journal state; no external effect is admitted or claimed exactly once. | `B-E` |
| Runtime adapter configuration parsing | [adapters](../architecture/runtime/adapters.md) | `crates/lkjmc-core/src/kubernetes.rs`; `crates/lkjmc-core/src/config/runtime_validate.rs` | `crates/lkjmc-core/src/kubernetes_tests.rs` | none | Parsing or selecting an adapter does not admit process or Kubernetes work; all external adapter effects are denied. | `B-E` |

## Boundary

This matrix does not claim runtime serialization, external adapter completion,
or external exactly-once behavior. Request replay and correlation are bounded
to the two named PostgreSQL desired-state commands and proofs.
