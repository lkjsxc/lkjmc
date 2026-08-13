# Product journeys

## Purpose

This index connects supported player and operator entrypoints to their truthful
surface boundary.

## Status

implemented

## Player journeys

| Goal | Current entry | Owner | State boundary |
| --- | --- | --- | --- |
| Read network documentation | `/menu`, `/docs`, or slot-8 token | GUI | Bundled content only; no daemon row or action is inferred. |
| Join a configured network | Velocity connection | Network | MOTD and tab-list presentation do not authorize a daemon action. |
| Buy, transfer, claim, or start an adventure | none in Java | Economy, Claims, Adventures | Java daemon adapters are withdrawn pending attestation. |

## Operator journeys

| Goal | Current entry | Owner | State boundary |
| --- | --- | --- | --- |
| Manage durable runtime state | CLI or authenticated web | Admin, Network | Daemon authorization and diagnostics are final. |
| Maintain catalog and temporary sessions | root-authorized daemon or CLI operation | Economy, Adventures | Durable facts do not establish a player delivery or transfer. |
| Moderate | authorized daemon operation | Social | Failed work is not reported as a mutation. |

## Shared state contract

A Java presentation surface is local-safe unless an owner document names trusted
identity/session attestation and implementation evidence. Missing player
authority is unavailable, not a disabled row, fallback mutation, or successful
smoke result.

## Evidence boundary

Rows labelled current are bounded by repository source and checks. External
outcomes need their separately named guarded evidence.
