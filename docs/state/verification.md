# Verification state

## Purpose

This matrix records available verification tiers and their evidence bounds.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Documentation and contract topology checks | [verification](../operations/verification.md) | `scripts/check-docs.py`; `scripts/check-doc-coverage.py`; `scripts/check-lines.py`; `scripts/check-menus.py` | `./scripts/check-lines.py`; `./scripts/check-docs.py`; `./scripts/check-doc-coverage.py` | none | These checks validate topology and bounded contracts, not runtime effects. | `D-VERIFY` |
| Fast and full local verification tiers | [CI](../operations/continuous-integration.md) | `scripts/verify-fast.sh`; `scripts/verify-full.sh`; `docker-compose.yml` | `./scripts/verify-fast.sh`; `./scripts/verify-full.sh`; `docker compose --profile verify run --rm verify` | none | Compose is deterministic DB verification, not external live proof. | `F-SAFE-OPS` |
| Supported opt-in external checks | [smoke checks](../operations/smoke-checks.md) | `scripts/verify-live.sh`; `scripts/check-bedrock-smoke.sh`; `scripts/check-discord-smoke.sh`; `scripts/check-kubernetes-smoke.sh` | `./scripts/verify-live.sh` reports declared skips | Guard variables in the owner document | A skipped supported lane is not passed; blocked Java paths are neither skips nor proof. | `P-DISCORD`, `P-KUBE` |
| Test-only seeded fault replay | [verification](../operations/verification.md) | `crates/lkjmc-daemon/src/fault_harness`; `scripts/check-fault-harness.py` | `scripts/check-fault-harness.py` | none | Injected-failure evidence is not release or product proof. | `F-FAULTS` |

## Boundary

No result is inferred from a prior run. Each claim needs the command, outcome,
and redacted evidence from the run being reported.
