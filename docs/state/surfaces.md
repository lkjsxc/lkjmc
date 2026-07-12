# Surface state

## Purpose

This matrix records bounded shipped adapter and operator-surface capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Local-safe Java presentation | [commands](../product/commands/minecraft.md) | `platforms/jvm/paper/src/main/java/com/lkjmc/paper/DocsCommandAdapter.java`; `platforms/jvm/velocity/src/main/java/com/lkjmc/velocity/VelocityMotdAdapter.java` | `scripts/check-jvm-containment.py` | none | Paper has only `/menu`, `/docs`, and hotbar/docs UI; Velocity has only MOTD/tab-list. | `F-SAFETY-GATE` |
| authenticated `/web` operator pages with session and scoped-credential handling | [web security](../architecture/web/security.md) | `crates/lkjmc-daemon/src/web/auth.rs`; `crates/lkjmc-daemon/src/web/sessions.rs` | `crates/lkjmc-daemon/src/tests/web_api_tests.rs`; `scripts/check-security-probes.py` | `LKJMC_WEB_SMOKE=1 ./scripts/check-web-smoke.sh` | The bootstrap secret creates a session only; scoped credentials fail closed when PostgreSQL revision verification is unavailable. | `A-SECURITY`, `F-SAFE-AUTH` |
| `kubernetes` selectable runtime and owned-object planning | [Kubernetes operations](../operations/kubernetes-runtime.md) | `crates/lkjmc-core/src/kubernetes.rs`; `crates/lkjmc-core/src/kubernetes_tests.rs` | `crates/lkjmc-core/src/kubernetes_tests.rs` | `LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` | The smoke does not prove logs, post-stop/delete state, or restart recovery. | `D-OPS`, `F-SAFE-RUNTIME` |
| Discord command withdrawal | [Discord](../product/discord/bot-service.md) | `crates/lkjmc-discord/src/commands.rs`; `crates/lkjmc-discord/src/interaction.rs` | `cargo test -p lkjmc-discord` | `LKJMC_DISCORD_SMOKE=1 ./scripts/check-discord-smoke.sh` | The guarded lane can remove prior registrations only; no Discord action is shipped. | `P-DISCORD`, `F-SAFE-AUTH` |

## Boundary

Java daemon adapters are withdrawn pending trusted identity/session attestation.
No state row claims a Java daemon command, dynamic menu, grant snapshot, token
refresh, proxy registration, or transfer as shipped.
