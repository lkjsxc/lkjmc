# Gameplay state

## Purpose

This matrix records bounded shipped player-data and Paper menu capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Immutable profile snapshots and session saves | [player profile](../product/sync/player-profile.md) | `crates/lkjmc-store/src/player.rs`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/PlayerProfileAdapter.java` | `crates/lkjmc-store/tests/player_session.rs` | `LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 ./scripts/check-playable-smoke.sh` | Recovery reporting records facts; it does not repair a profile automatically. | `A-DATA`, `F-SAFE-ECON` |
| Claim storage and asynchronous Paper protection reads | [claims](../product/claims/README.md) | `crates/lkjmc-store/src/claims.rs`; `platforms/jvm/common/src/main/java/com/lkjmc/common/claim/ClaimProtectionPolicy.java` | `platforms/jvm/common/src/test/java/com/lkjmc/common/claim/ClaimProtectionPolicyTest.java` | `LKJMC_MINECRAFT_CLAIM_SMOKE=1 ./scripts/check-minecraft-claim-smoke.sh` | Live protocol behavior needs the separately guarded claim protocol lane. | `F-CLAIM-PROBES`, `A-PRODUCT` |
| Document-driven Paper menus | [menu engine](../architecture/plugin/menu-engine.md) | `platforms/jvm/common/src/main/java/com/lkjmc/common/ui/kernel/UiUpdate.java`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/ui/UiSessionService.java` | `platforms/jvm/common/src/test/java/com/lkjmc/common/ui/kernel/UiUpdateBehaviorTest.java`; `scripts/check-menus.py` | `LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 ./scripts/check-playable-smoke.sh` | Velocity does not use this engine; guarded smoke is required for player-visible delivery. | `A-MENU`, `F-SAFE-JVM` |
| Correlation-safe economy settlement | [economy](../product/economy/README.md) | `crates/lkjmc-store/src/shop.rs`; `crates/lkjmc-store/src/exchange.rs`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/ShopCommandAdapter.java`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/ExchangeCommandAdapter.java` | PostgreSQL-gated store replay tests; Paper adapter replay tests | `LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 ./scripts/check-playable-smoke.sh` | Transport ambiguity and incomplete inventory restoration are contained for reconciliation; guarded smoke alone proves a live delivery. | `F-SAFE-ECON` |

## Boundary

Unlisted economy, social, adventure, and target features are not shipped claims
in this state summary.
