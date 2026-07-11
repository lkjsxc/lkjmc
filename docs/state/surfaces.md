# Surface state

## Purpose

This matrix records bounded shipped adapter and operator-surface capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Local-safe Java presentation | [commands](../product/commands/minecraft.md) | `platforms/jvm/paper/src/main/java/com/lkjmc/paper/DocsCommandAdapter.java`; `platforms/jvm/velocity/src/main/java/com/lkjmc/velocity/VelocityMotdAdapter.java` | `scripts/check-jvm-containment.py` | none | Paper has only `/menu`, `/docs`, and hotbar/docs UI; Velocity has only MOTD/tab-list. | `F-SAFETY-GATE` |
| authenticated `/web` operator pages with session handling | [web routes](../architecture/web/routes.md) | `crates/lkjmc-daemon/src/web/routes.rs`; `crates/lkjmc-daemon/src/web/sessions.rs` | `crates/lkjmc-daemon/src/tests/web_api_tests.rs` | `LKJMC_WEB_SMOKE=1 ./scripts/check-web-smoke.sh` | The browser path is skipped without its guard and prerequisites. | `A-SECURITY`, `F-SAFE-AUTH` |
| `kubernetes` selectable runtime and owned-object planning | [Kubernetes operations](../operations/kubernetes-runtime.md) | `crates/lkjmc-core/src/kubernetes.rs`; `crates/lkjmc-core/src/kubernetes_tests.rs` | `crates/lkjmc-core/src/kubernetes_tests.rs` | `LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` | The smoke does not prove logs, post-stop/delete state, or restart recovery. | `D-OPS`, `F-SAFE-RUNTIME` |
| Discord command withdrawal | [Discord](../product/discord/bot-service.md) | `crates/lkjmc-discord/src/commands.rs`; `crates/lkjmc-discord/src/interaction.rs` | `cargo test -p lkjmc-discord` | `LKJMC_DISCORD_SMOKE=1 ./scripts/check-discord-smoke.sh` | The guarded lane can remove prior registrations only; no Discord action is shipped. | `P-DISCORD`, `F-SAFE-AUTH` |

## Boundary

Java daemon adapters are withdrawn pending trusted identity/session attestation.
No state row claims a Java daemon command, dynamic menu, grant snapshot, token
refresh, proxy registration, or transfer as shipped.
