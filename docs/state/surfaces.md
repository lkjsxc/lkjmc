# Surface state

## Purpose

This matrix records bounded shipped adapter and operator-surface capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Shared `/lkjmc` command metadata on Java adapters | [commands](../product/commands/README.md) | `platforms/jvm/common/src/main/java/com/lkjmc/common/command/LkjmcCommandTree.java`; `platforms/jvm/velocity/src/main/java/com/lkjmc/velocity/VelocityLkjmcCommand.java` | `platforms/jvm/common/src/test/java/com/lkjmc/common/command/LkjmcCommandTreeTest.java`; `platforms/jvm/velocity/src/test/java/com/lkjmc/velocity/VelocityLkjmcCommandTest.java` | `LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 ./scripts/check-playable-smoke.sh` | Unit proof does not prove a real client command session. | `A-JVM`, `A-NETWORK` |
| authenticated `/web` operator pages with session handling | [web routes](../architecture/web/routes.md) | `crates/lkjmc-daemon/src/web/routes.rs`; `crates/lkjmc-daemon/src/web/sessions.rs` | `crates/lkjmc-daemon/src/tests/web_api_tests.rs` | `LKJMC_WEB_SMOKE=1 ./scripts/check-web-smoke.sh` | The browser path is skipped without its guard and prerequisites. | `A-SECURITY`, `F-SAFE-AUTH` |
| `kubernetes` selectable runtime and owned-object planning | [Kubernetes operations](../operations/kubernetes-runtime.md) | `crates/lkjmc-core/src/kubernetes.rs`; `crates/lkjmc-core/src/kubernetes_tests.rs` | `crates/lkjmc-core/src/kubernetes_tests.rs` | `LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh` | The smoke does not prove logs, post-stop/delete state, or restart recovery. | `D-OPS`, `F-SAFE-RUNTIME` |
| Discord request delegation | [Discord](../product/discord/bot-service.md) | `crates/lkjmc-discord/src/discord_api.rs`; `crates/lkjmc-daemon/src/commands/discord_api.rs` | `cargo test -p lkjmc-discord` | `LKJMC_DISCORD_SMOKE=1 ./scripts/check-discord-smoke.sh` | Registration and signed live interaction proof require real credentials. | `P-DISCORD`, `A-NETWORK` |

## Boundary

This state does not claim Bedrock connectivity or any guarded surface as live
proved without the named guarded command and its evidence.
