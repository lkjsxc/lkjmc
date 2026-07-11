# Incident response

## Purpose

This runbook defines a common, evidence-first response for operator incidents.

## Status

implemented

## First response

Protect people and evidence first: do not print secrets, mutate production while
triaging, or call a skipped check a pass. Record UTC time, affected instance or
surface, operator, redacted command output, and the last known healthy state.
Use a dedicated incident record outside this repository.

## Incident matrix

| Symptom | Contain | Inspect | Resolution boundary |
| --- | --- | --- | --- |
| Daemon unavailable | Stop dependent mutations | service status, daemon logs | Boot and doctor succeed on intended config |
| Token rejection | Do not copy token bytes | token-file paths, safe auth logs | New token accepted; old token rejected |
| Database error | Stop writes if integrity is uncertain | `lkjmc db status`, PostgreSQL logs | Isolated restore validation passes |
| Backend unavailable | Hold transfers to that backend | instance list, doctor, bounded logs | Adapter reports healthy and route is checked |
| Kubernetes failure | Restrict action to its namespace | labeled pods and events | Observation matches deliberate action |
| Suspected secret exposure | Revoke affected access | audit and redacted logs | Rotation and access review are recorded |

## Escalation and proof

Collect only redacted logs and bounded command output. Preserve the configured
paths, artifact identity, and external observations without secret values. A
symptom disappearing is not resolution: record the exact validation that proves
the stated resolution boundary. Use [backup and restore](backup-restore.md) for
database recovery and [lifecycle and recovery](lifecycle-recovery.md) for
runtime adoption limits.
