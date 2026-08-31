# Product journeys

## Player journeys

| Goal | Current entry | Owner | Evidence state |
| --- | --- | --- | --- |
| Connect to the configured Java entrypoint | selected Velocity listener | Velocity | Requires exact-environment protocol/client evidence; real-player acceptance is separate. |
| Read local guidance | `/menu`, `/docs`, or slot token | Paper/Folia | Java-tested; real-player click not currently claimed. |
| Inspect backend state | `/lkjmc status` | Velocity | Dynamic inventory and bounds tested; real-player output not currently claimed. |
| Move between backends | `/lkjmc server <instance-id>` | Velocity | Connection-request outcomes tested; same-player arrival not currently claimed. |

Instance IDs in examples are data, not journeys or roles. Broad gameplay, economy, moderation,
Discord, Bedrock, Kubernetes, and public web behavior remain outside the narrow supported core unless
their own owner and evidence say otherwise.

## Operator journeys

The protected Rust CLI and daemon own desired-state inspection and lifecycle effects. `lkjmc-ops`
owns anchored update, recovery, EULA materialization, backup/restore verification, fence handling,
post-start acceptance, and bounded diagnosis for the dynamic fleet. A dormant command shard or
handler is not a supported journey.

## Evidence boundary

Source implementation, deterministic tests, PostgreSQL proof, packaged bytes, installation,
protocol-client observation, real-player observation, and production observation are distinct.
