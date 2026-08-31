# Incident response

## Purpose

Define evidence-first diagnosis, containment, and recovery commands.

## Status

implemented

## First response

Protect people and evidence: do not print secrets, mutate uncertain production,
or call a skip a pass. Record UTC time, commit and artifact-manifest checksum,
affected surface, operator, external prerequisites, exact command exits, bounded
redacted output, and last known healthy observation in a private incident record.

## Command matrix

| Symptom | Contain | Execute and retain | Recovery proof |
| --- | --- | --- | --- |
| daemon unavailable | stop dependent mutations | `systemctl status lkjmc-daemon`; `lkjmc status --json` | socket readiness and status query pass |
| token rejection | revoke, never copy token bytes | inspect private token path/mode and bounded auth events | new token accepted; old token denied |
| database error | stop writers | `lkjmc db status`; bounded PostgreSQL logs; `lkjmc-ops database backup ...` | isolated restore verification and accepted service pass |
| corrupt backup | quarantine all members | `sha256sum --check incident.dump.sha256`; restore to fresh DB | corruption rejects; prior backup boots |
| backend loss | hold transfers | `lkjmc instance list`; `lkjmc doctor`; bounded instance logs | real adapter readiness is observed |
| disk pressure | stop downloads/writes | `df -P`; private roots and partial files | space is reclaimed and partial count is zero |
| network loss | stop cutover | loopback readiness, listener ownership, authorized route probe | intended listener and route are observed |
| suspected exposure | revoke affected access | private audit/support bundle with canary scan | rotation and access review recorded |

Every command runs once. A diagnostic failure is retained, not retried away.
Generated support output uses `lkjmc support bundle --output PATH`; the archive,
parent, and members remain private and must pass the final canary scan.

## Fault rehearsal

Focused Rust tests cover lock contention, partial publication, owned subprocess bounds, fence and
permit replay, pre-ledger rollback, changed-ledger blocking, backup corruption, and idempotent
recovery. Required CI adds real PostgreSQL transactions and isolated schema state. These are
deterministic/process/PostgreSQL evidence, not authority to inject production faults or claims about
real systemd and Minecraft.

## External prerequisites and escalation

Cloud database controls, firewall/DNS/load balancers, Kubernetes namespaces,
Minecraft clients, public endpoints, provider credentials, signing keys, and
production log archives require explicit authorization and are otherwise skips.
Escalate when containment cannot stop writes, integrity is uncertain, a secret
may have escaped, ownership is ambiguous, cleanup fails, or rollback proof does
not pass. A symptom disappearing is never resolution evidence.
