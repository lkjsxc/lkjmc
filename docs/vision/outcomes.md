# Outcomes and quality goals

## Purpose

This document defines target outcomes and the quality goals that make them
credible. It does not declare any outcome shipped.

## Target outcomes

| Person | Target outcome | Owner areas | Quality goal |
| --- | --- | --- | --- |
| Operator | Install, inspect, and change a network without guessing its durable or runtime state. | [Operations](../operations/README.md), [Architecture](../architecture/README.md), [Product admin](../product/admin/README.md) | Every mutation returns a durable result or diagnostic; authorization and audit remain authoritative. |
| Player | Enter a ready Java network, find a useful activity, and receive truthful localized feedback. | [Product](../product/README.md), [Network](../product/network/README.md), [GUI](../product/gui/README.md) | An unavailable action is disabled or fails explicitly; no balance, transfer, or success is invented. |
| Agent | Make a bounded, reviewable change with owner documentation, deterministic checks, and an honest handoff. | [Agent](../agent/README.md), [Repository](../repository/README.md), [Operations verification](../operations/verification.md) | Contracts precede behavior; evidence names what ran, skipped, and remains externally blocked. |

## Quality goals

| Goal | Target measure | Accountable owner |
| --- | --- | --- |
| Truthful control | Each registered mutation has a real effect path and typed failure boundary. | Architecture and Product |
| Durable consistency | Product truth has one PostgreSQL owner; adapters do not create competing state. | Architecture data |
| Safe responsiveness | Minecraft scheduler callbacks do not wait on database, filesystem, network, or process work. | Architecture plugin |
| Recoverability | Failed or uncertain effects expose a diagnostic or recovery record, never a fabricated completion. | Architecture orchestration and Operations |
| Verifiability | Deterministic checks prove contracts; external proof is guarded and reports skips exactly. | Operations verification and Research |
| Usability | Player and operator paths state loading, empty, disabled, and recovery conditions in localized surfaces. | Product GUI, I18n, and Admin |

## Non-goals

`lkjmc` is not a decorative command or menu collection, a second plugin-local
product database, an unattended public control panel, or a promise to operate
any external service without its credentials and prerequisites. It does not
make every Minecraft server profile-sync capable, replace Minecraft gameplay
content, or treat an agent's plan as authorization to make effects.

## Current boundary

Current behavior is owned only by [state](../state/README.md) and its exact
implementation and verification evidence. This target supplies selection
criteria for future work; it must not be read as evidence that a target outcome
is available in a particular deployment.
