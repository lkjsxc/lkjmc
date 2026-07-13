# Gameplay state

## Purpose

This matrix records bounded shipped Paper presentation behavior.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Local bundled-document Paper menu and hotbar token | [GUI](../product/gui/README.md) | `platforms/jvm/paper/src/main/java/com/lkjmc/paper/LocalDocsMenu.java`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/HotbarMenuListener.java` | `platforms/jvm/paper/src/test/java/com/lkjmc/paper/LocalPaperSurfaceTest.java`; `scripts/check-jvm-containment.py` | none | No daemon data, action, profile, claim, economy, transfer, or admin route is active. | `F-SAFETY-GATE` |
| Typed profile and durable workflow intent | [player profile](../product/sync/player-profile.md); [transfer safety](../product/sync/transfer-safety.md) | `crates/lkjmc-core/src/profile_validation.rs`; `crates/lkjmc-store/src/data_workflows`; `crates/lkjmc-store/src/player_session.rs`; `migrations/045-durable-data-workflows.sql` | `scripts/check-data-workflows.py`; `crates/lkjmc-store/tests/data_workflows.rs`; `crates/lkjmc-store/tests/player_session.rs` | none | Session joins roll back as one store transaction; feed cursors below the active floor require reload. PostgreSQL intent and failure facts do not prove player, inventory, runtime, or cleanup effects; Java bridges remain absent. | `A-SYNC`, `A-RUNTIME` |

## Boundary

Durable player data, claims, economy, social, travel, and adventure APIs do not
make a Paper player surface available without trusted adapter attestation.
