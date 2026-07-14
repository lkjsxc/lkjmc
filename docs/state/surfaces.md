# Surface state

## Purpose

This matrix records bounded shipped adapter and operator-surface capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Typed Java menu presentation | [commands](../product/commands/minecraft.md) | `platforms/jvm/paper/src/main/java/com/lkjmc/paper/PaperMenuAdapter.java`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/MenuResponseOwnership.java`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/PaperMenuProtocolAdapter.java`; `platforms/jvm/velocity/src/main/java/com/lkjmc/velocity/VelocityMotdAdapter.java` | `platforms/jvm/paper/src/test/java/com/lkjmc/paper/harness/MenuProbeRunner.java`; `scripts/check-jvm-containment.py` | none | Paper registers only `/menu` and `/docs`; typed menu/permission/claim/settings views are read-only and mutation denial is final without attestation and a typed port. Velocity has MOTD/tab-list only. | guarded Minecraft lane |
| Revisioned read-only Java transport | [revisioned transport](../product/sync/revisioned-transport.md) | `crates/lkjmc-daemon/src/transport/sync.rs`; `platforms/jvm/common/src/main/java/com/lkjmc/common/sync/SyncCoordinator.java`; `migrations/047-revisioned-sync.sql` | `scripts/check-sync-adoption.py`; `tests/test_sync_adoption_checker.py` | none | Scoped loopback reads and immutable cache views do not apply player data, authorize actions, mutate state, or transfer players. | `A-JVM` |
| authenticated `/web` operator pages with session and scoped-credential handling | [web security](../architecture/web/security.md) | `crates/lkjmc-daemon/src/web/auth.rs`; `crates/lkjmc-daemon/src/web/routes.rs`; `crates/lkjmc-daemon/src/web/sessions.rs` | web and lifecycle saturation/containment tests; `scripts/check-security-probes.py` | `LKJMC_WEB_SMOKE=1 ./scripts/check-web-smoke.sh` | The bootstrap secret creates a session only; scoped credentials fail closed when PostgreSQL revision verification is unavailable. Web auth, audit, rendering, and dispatch share the eight-lease deadline boundary. | `A-SECURITY`, `F-SAFE-AUTH` |
| `kubernetes` selectable runtime parsing and pure planning | [runtime adapters](../architecture/runtime/adapters.md) | `crates/lkjmc-core/src/kubernetes.rs`; `crates/lkjmc-core/src/kubernetes_tests.rs` | `crates/lkjmc-core/src/kubernetes_tests.rs`; `scripts/check-command-lifecycle.py` (effect-classes-enforced probe) | none | Selection and plan validation do not apply, observe, log, stop, delete, or recover an object; every adapter effect is denied. | `B-E` |
| Discord command withdrawal | [Discord](../product/discord/bot-service.md) | `crates/lkjmc-discord/src/config.rs`; `crates/lkjmc-discord/src/commands.rs` | `cargo test -p lkjmc-discord`; lifecycle Discord-boundary probe | `LKJMC_DISCORD_SMOKE=1 ./scripts/check-discord-smoke.sh` | The guarded lane can remove prior registrations only; `interactionBind` fails closed and no Discord action or listener is shipped. | `P-DISCORD`, `F-SAFE-AUTH` |
| Bounded JVM and Discord local diagnostics | [observability](../architecture/runtime/daemon/observability.md) | `platforms/jvm/common/src/main/java/com/lkjmc/common/diagnostic/`; `crates/lkjmc-discord/src/diagnostics.rs` | JVM diagnostic emitter tests; `cargo test -p lkjmc-discord`; `scripts/check-observability.py` | none | Typed diagnostics enqueue without callback blocking, reject sensitive values, and drain with deadlines. They are adapter-local output, not independently attested events. | `B-O` |

## Boundary

The Java menu consumes cached typed read views but no state row claims a Java
daemon command, authorization grant, player mutation, purchase, claim change,
proxy registration, application, or transfer as shipped.
