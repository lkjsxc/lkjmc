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
| database error | stop writers | `lkjmc db status`; PostgreSQL bounded logs; `scripts/backup-postgres.sh incident.dump` | fresh restore/boot drill passes |
| corrupt backup | quarantine all members | `sha256sum --check incident.dump.sha256`; restore to fresh DB | corruption rejects; prior backup boots |
| backend loss | hold transfers | `lkjmc instance list`; `lkjmc doctor`; bounded instance logs | real adapter readiness is observed |
| disk pressure | stop downloads/writes | `df -P`; private roots and partial files | space is reclaimed and partial count is zero |
| network loss | stop cutover | loopback readiness, listener ownership, authorized route probe | intended listener and route are observed |
| suspected exposure | revoke affected access | private audit/support bundle with canary scan | rotation and access review recorded |

Every command runs once. A diagnostic failure is retained, not retried away.
Generated support output uses `lkjmc support bundle --output PATH`; the archive,
parent, and members remain private and must pass the final canary scan.

## Fault rehearsal

`scripts/run-operations-lab.py` invokes real PostgreSQL transaction/restore,
filesystem partial-write, owned child-process, and loopback network/readiness
boundaries with a recorded seed. It preserves redacted artifacts before cleanup
and proves no child, socket, database, Compose resource, or partial final file
survives. This is lab evidence, not authority to inject production faults.

## External prerequisites and escalation

Cloud database controls, firewall/DNS/load balancers, Kubernetes namespaces,
Minecraft clients, public endpoints, provider credentials, signing keys, and
production log archives require explicit authorization and are otherwise skips.
Escalate when containment cannot stop writes, integrity is uncertain, a secret
may have escaped, ownership is ambiguous, cleanup fails, or rollback proof does
not pass. A symptom disappearing is never resolution evidence.
