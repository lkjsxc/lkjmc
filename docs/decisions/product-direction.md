# Product direction decisions

## Purpose

This document fixes product-direction constraints and records provisional
choices that must be governed by experiments rather than taste.

## Fixed constraints

- PostgreSQL is the sole durable product store; plugins and external adapters
  do not become competing sources of truth.
- Rust pure cores decide validation and plans; effect adapters perform database,
  filesystem, network, process, and cluster work.
- Minecraft scheduler callbacks never block on an effect boundary.
- A registered control must make a real request and return a real result,
  disabled reason, or failure; fake success is prohibited.
- Privileged mutations require authorization, safe auditing, and redacted
  diagnostics. Generated secrets are never printed.
- User-edited configuration is JSON. External prerequisites remain explicit and
  guarded proof reports a skip rather than a pass.

These constraints refine [architecture](../architecture/README.md),
[security](../architecture/security/README.md), and
[operations](../operations/README.md); they are product choices, not proof of
current implementation.

## Experiment-governed choices

| Choice | Provisional decision | Experiment and exit rule | Owner |
| --- | --- | --- | --- |
| Default player entry | Keep Java play as the required path; treat Bedrock compatibility as optional. | Compare guarded Java and Bedrock entry evidence; promote no broader claim until a supported endpoint and live smoke succeed. | Product network and Research |
| Wake behavior | Let a player request wake only through a truthful status-bearing flow. | Measure request completion, expiry, cancellation, and failed transfer handling; change defaults only with durable evidence. | Product travel and Architecture orchestration |
| Profile scope | Keep location opt-in and profile sync limited to supported plugin-enabled servers. | Exercise transfer recovery and privacy review; expand fields only with schema, consent, and failure evidence. | Product sync and Architecture data |
| Operator surface | Prefer CLI and private authenticated presentation adapters over a public control plane. | Evaluate authorized mutation, audit, CSRF, and guarded live-smoke evidence; reject any path that weakens them. | Operations and Architecture web |
| Menu entry | Keep the hotbar entry opt-in while command and menu paths coexist. | Observe discoverability and accidental activation through an approved study; retain opt-in unless a documented benefit outweighs it. | Product GUI |

Each experiment needs a written owner plan, reversible configuration or rollout,
measurements, a failure boundary, and an explicit accept, reject, or extend
decision. Lack of data retains the provisional choice; it does not block a
safe implementation already covered by fixed constraints.

## Current boundary

This is a target decision record. [State](../state/README.md) remains the sole
source for whether a capability is currently implemented, and
[research](../research/README.md) owns questions that have not reached an owner
contract.
