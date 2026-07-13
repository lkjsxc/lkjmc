# Control plane state

## Purpose

This matrix records bounded shipped Rust control-plane capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Closed command-member validation and fail-closed effect classes | [registry](../contracts/command-registry.md) | `contracts/commands/README.json`; `crates/lkjmc-core/src/command_registry.rs`; `crates/lkjmc-daemon/src/command_lifecycle.rs` | `crates/lkjmc-daemon/src/command_lifecycle.rs`; `scripts/check-contracts.py`; `scripts/check-command-lifecycle.py` (effect-classes-enforced probe) | none | 137 contracts classify as 1 local observation, 2 PostgreSQL reads, 2 desired-state writes, 1 restart requirement, or 131 denials. | `B-E` |
| Bounded daemon request admission and desired-state outcomes | [lifecycle](../architecture/runtime/daemon/command-lifecycle.md) | `crates/lkjmc-daemon/src/app/`; `crates/lkjmc-daemon/src/transport/`; `crates/lkjmc-daemon/src/dispatch.rs`; `crates/lkjmc-store/src/command.rs`; `crates/lkjmc-store/src/pool.rs` | `scripts/check-command-lifecycle.py`; `crates/lkjmc-daemon/src/app/admission/tests.rs`; `crates/lkjmc-daemon/src/tests/deadline_route_tests.rs`; `crates/lkjmc-daemon/src/tests/command_operation_tests/replay.rs`; `crates/lkjmc-daemon/src/tests/command_operation_tests/timeout.rs`; `crates/lkjmc-daemon/src/tests/command_operation_tests/failure.rs` | Compose verify runs required real-PostgreSQL timeout and replay probes. | Eight shared leases cover auth, denial audit, commands, and web paths. Journal-admitted desired-state mutations retain request correlation and finish in a queryable succeeded, failed, or cancelled state while PostgreSQL remains reachable; no external effect is admitted or claimed exactly once. | `B-E` |
| Fenced per-instance runtime lifecycle | [adapters](../architecture/runtime/adapters.md); [desired state](../architecture/orchestration/desired-state.md) | `crates/lkjmc-daemon/src/runtime/`; `crates/lkjmc-store/src/runtime_adoption/`; `migrations/046-runtime-adoption.sql` | `scripts/check-runtime-adoption.py` (seven named probes) | `scripts/check-kubernetes-smoke.sh` when explicitly guarded | Local effects use keyed serialization, durable fence ownership, identity-bound processes, bounded recovery, and append-only attempts. Kubernetes capability denial and planning are deterministic; live cluster effects remain external proof. | A live Kubernetes run may skip only for its named prerequisite and is not local capability proof. | `A-RUNTIME` |

## Boundary

This matrix does not claim runtime serialization, external adapter completion,
or external exactly-once behavior. Request replay and correlation are bounded
to the two named PostgreSQL desired-state commands and proofs.
