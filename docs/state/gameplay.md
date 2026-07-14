# Gameplay state

## Purpose

This matrix records bounded shipped Paper presentation behavior.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Document-driven Paper menu and hotbar token | [GUI](../product/gui/README.md) | `platforms/jvm/common/src/main/java/com/lkjmc/common/menu`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/PaperMenuAdapter.java`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/MenuResponseOwnership.java`; `platforms/jvm/common/src/generated/resources/lkjmc-menu-bundle.json` | `platforms/jvm/paper/src/test/java/com/lkjmc/paper/harness/MenuProbeRunner.java`; `platforms/jvm/paper/src/test/java/com/lkjmc/paper/harness/MenuMutationTest.java`; `scripts/check-menus.py`; `scripts/check-jvm-containment.py` | none | All 62 routes render current, stale, or unavailable views; stale async responses lose ownership and perform no UI or chat effect. Mutations have no daemon port and deny without current capability plus attestation. Protocol evidence is not a live player. | guarded Minecraft lane |
| Typed profile and durable workflow intent | [player profile](../product/sync/player-profile.md); [transfer safety](../product/sync/transfer-safety.md) | `crates/lkjmc-core/src/profile_validation.rs`; `crates/lkjmc-store/src/data_workflows`; `crates/lkjmc-store/src/player_session.rs`; `migrations/045-durable-data-workflows.sql` | `scripts/check-data-workflows.py`; `crates/lkjmc-store/tests/data_workflows.rs`; `crates/lkjmc-store/tests/player_session.rs` | none | Session joins roll back as one store transaction; feed cursors below the active floor require reload. PostgreSQL intent and failure facts do not prove player, inventory, runtime, or cleanup effects; Java bridges remain absent. | `A-SYNC`, `A-RUNTIME` |

## Boundary

Typed snapshots can render bounded Paper views. They do not authorize a player
mutation, transfer, purchase, claim change, or adventure start; those effects
still require trusted attestation and an implemented typed port.
