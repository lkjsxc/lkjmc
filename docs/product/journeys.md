# Product journeys

## Player journeys

| Goal | Current entry | Owner | Evidence state |
| --- | --- | --- | --- |
| Connect to the network | Velocity `25591` | Velocity | Protocol status ping observed; authorized login still unavailable. |
| Read local guidance and docs | `/menu`, `/docs`, or slot-8 token | Paper/Folia | Candidate adapter tested; real player click not yet observed. |
| Inspect network state | `/lkjmc status` | Velocity | Parsing and registration tested; real player output not yet observed. |
| Move between backends | `/lkjmc server hub|survival` | Velocity | Transfer path tested without a real authorized player; live transfer outstanding. |

No economy, claim, home, mail, party, adventure, moderation, or admin inventory
journey is currently supported.

## Operator journeys

The protected local CLI owns health, status, logs, start, stop, restart,
reconcile/bootstrap, backup, and restore. The daemon Unix socket and loopback
plugin heartbeat endpoint are private boundaries. A dormant command shard or
handler is not a supported journey.

## Evidence boundary

Implementation, deterministic tests, deployed bytes, registration, and a real
client observation are distinct states. This document does not treat an
installed command or rendered candidate frame as player acceptance.
