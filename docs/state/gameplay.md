# Gameplay state

## Purpose

This matrix records bounded shipped player-data and local Paper menu capabilities.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Typed profile and claim storage | [player profile](../product/sync/player-profile.md) | `crates/lkjmc-store/src/player.rs`; `crates/lkjmc-store/src/claims.rs` | `crates/lkjmc-store/tests/player_session.rs`; `crates/lkjmc-store/tests/claims.rs` | none | No Paper profile save, claim command, or live protection refresh is shipped. | `F-SAFETY-GATE` |
| Local document-driven Paper menu | [menu engine](../architecture/plugin/menu-engine.md) | `platforms/jvm/common/src/main/java/com/lkjmc/common/docs/DocBundle.java`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/DocsCommandAdapter.java` | `scripts/check-jvm-containment.py`; `scripts/check-menus.py` | none | It renders bundled docs only; no daemon row, grant snapshot, or mutation exists. | `F-SAFETY-GATE` |

## Boundary

Economy, social, adventure, profile, claim, and admin Java adapters are
withdrawn pending trusted identity/session attestation. Daemon/store capability
does not claim a Java player surface.
