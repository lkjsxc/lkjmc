# Gameplay state

## Purpose

This matrix records bounded shipped player-data and Paper menu capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Typed profile snapshots and session saves | [player profile](../product/sync/player-profile.md) | `crates/lkjmc-store/src/player.rs`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/PlayerProfileAdapter.java` | `./scripts/check-jvm-safety.py`; `crates/lkjmc-store/tests/player_session.rs` | `LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 ./scripts/check-playable-smoke.sh` | Recovery reporting records facts; it does not repair a profile automatically. | `A-DATA`, `F-SAFE-ECON` |
| Claim storage and asynchronous Paper protection reads | [claims](../product/claims/README.md) | `crates/lkjmc-store/src/claims.rs`; `platforms/jvm/common/src/main/java/com/lkjmc/common/claim/ClaimProtectionPolicy.java` | `platforms/jvm/common/src/test/java/com/lkjmc/common/claim/ClaimProtectionPolicyTest.java` | `LKJMC_MINECRAFT_CLAIM_SMOKE=1 ./scripts/check-minecraft-claim-smoke.sh` | Live protocol behavior needs the separately guarded claim protocol lane. | `F-CLAIM-PROBES`, `A-PRODUCT` |
| Correlated document-driven Paper menus | [menu engine](../architecture/plugin/menu-engine.md) | `platforms/jvm/common/src/main/java/com/lkjmc/common/ui/kernel/UiUpdate.java`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/ui/UiSessionService.java` | `platforms/jvm/common/src/test/java/com/lkjmc/common/ui/kernel/UiSafetyContainmentTest.java`; `scripts/check-menus.py`; `scripts/check-jvm-safety.py` | `LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 ./scripts/check-playable-smoke.sh` | Velocity does not use this engine; guarded smoke is required for player-visible delivery. | `A-MENU`, `F-SAFE-JVM` |

## Boundary

Unlisted economy, social, adventure, and target features are not shipped claims
in this state summary.
