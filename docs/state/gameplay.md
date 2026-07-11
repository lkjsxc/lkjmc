# Gameplay state

## Purpose

This matrix records bounded shipped Paper presentation behavior.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Local bundled-document Paper menu and hotbar token | [GUI](../product/gui/README.md) | `platforms/jvm/paper/src/main/java/com/lkjmc/paper/LocalDocsMenu.java`; `platforms/jvm/paper/src/main/java/com/lkjmc/paper/HotbarMenuListener.java` | `scripts/check-menus.py`; `scripts/check-jvm-containment.py` | none | No daemon data, action, profile, claim, economy, transfer, or admin route is active. | `F-SAFETY-GATE` |

## Boundary

Durable player data, claims, economy, social, travel, and adventure APIs do not
make a Paper player surface available without trusted adapter attestation.
