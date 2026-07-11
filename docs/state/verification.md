# Verification state

## Purpose

This matrix records the available verification tiers and their evidence bounds.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Documentation and contract topology checks | [verification](../operations/verification.md) | `scripts/check-docs.py`; `scripts/check-doc-coverage.py`; `scripts/check-lines.py`; `scripts/check-menus.py` | `./scripts/check-lines.py`; `./scripts/check-docs.py`; `./scripts/check-doc-coverage.py` | none | These checks validate documentation structure, coverage, constrained proof grammar, and contracts, not runtime effects. | `D-VERIFY` |
| Fast and full local verification tiers | [CI](../operations/continuous-integration.md) | `scripts/verify-fast.sh`; `scripts/verify-full.sh`; `compose.yaml` | `./scripts/verify-fast.sh`; `./scripts/verify-full.sh`; `docker compose --profile verify run --rm verify` | none | Compose is environment-backed deterministic verification, not an external live run. | `F-SAFE-OPS` |
| Opt-in Minecraft, web, Discord, and Kubernetes checks | [smoke checks](../operations/smoke-checks.md) | `scripts/verify-live.sh`; `scripts/check-playable-smoke.sh`; `scripts/check-web-smoke.sh`; `scripts/check-discord-smoke.sh`; `scripts/check-kubernetes-smoke.sh` | `./scripts/verify-live.sh` reports declared skips | Guard variables and commands listed in the owner document | A skipped guarded lane is skipped, never passed; Kubernetes coverage is intentionally narrow. | `P-PLAYABLE`, `P-DISCORD`, `P-KUBE` |

## Boundary

No result is inferred from a prior run. Each claim needs the command, outcome,
and redacted evidence from the run that is being reported.
