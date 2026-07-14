# Verification state

## Purpose

This matrix records available verification tiers and their evidence bounds.

## Status

implemented

## Capability matrix

| Capability | Owner document | Exact source | Deterministic proof | Guarded live proof | Present limit | Follow-up |
| --- | --- | --- | --- | --- | --- | --- |
| Documentation and contract topology checks | [verification](../operations/verification.md) | `scripts/check-docs.py`; `scripts/check-doc-coverage.py`; `scripts/check-lines.py`; `scripts/check-menus.py` | `./scripts/check-lines.py`; `./scripts/check-docs.py`; `./scripts/check-doc-coverage.py` | none | These checks validate topology and bounded contracts, not runtime effects. | `D-VERIFY` |
| Fast and full local verification tiers | [CI](../operations/continuous-integration.md) | `scripts/verify-fast.sh`; `scripts/verify-full.sh`; `scripts/check-db-test-isolation.sh`; `docker-compose.yml` | `./scripts/verify-fast.sh`; `./scripts/verify-full.sh`; `docker compose --profile verify run --rm verify` | none | Compose runs normal Cargo parallelism plus repeated four-thread isolated-schema DB tests; it is not external live proof. | `F-SAFE-OPS` |
| Supported opt-in external checks | [smoke checks](../operations/smoke-checks.md) | `scripts/verify-live.sh`; `scripts/check-bedrock-smoke.sh`; `scripts/check-discord-smoke.sh`; `scripts/check-kubernetes-smoke.sh` | `./scripts/verify-live.sh` reports declared skips | Guard variables in the owner document | A skipped supported lane is not passed; blocked Java paths are neither skips nor proof. | `P-DISCORD`, `P-KUBE` |
| Test-only seeded fault replay | [verification](../operations/verification.md) | `crates/lkjmc-daemon/src/fault_harness`; `scripts/check-fault-harness.py` | `scripts/check-fault-harness.py` | none | Injected-failure evidence is not release or product proof. | `F-FAULTS` |
| Durable data workflow classification and crash probes | [data architecture](../architecture/data/README.md) | `config/data-workflows.json`; `scripts/check-data-workflows.py` | eight named probes under `scripts/check-data-workflows.py`; `tests/test_data_workflow_checker.py` | Compose PostgreSQL is deterministic integration proof, not an external effect | A database probe without a valid URL fails; aggregate local verification may report it skipped but cannot call it passed. | `A-SYNC`, `A-RUNTIME` |
| Revisioned sync adoption | [revisioned transport](../product/sync/revisioned-transport.md) | `scripts/check-sync-adoption.py`; standalone Java 21 HTTP harness | eight exact probes; `tests/test_sync_adoption_checker.py` | none | PostgreSQL and Java prerequisites fail named probes; the harness proves transport behavior, not Minecraft application or transfer. | `A-JVM` |
| Network adoption and recovery probes | [verification](../operations/verification.md) | `scripts/check-network-adoption.py`; `scripts/network_adoption_checks.py`; `tests/test_network_adoption_checker.py` | six exact probes; `tests/test_network_adoption_checker.py` | none | Database probes require PostgreSQL. Local effects use disposable files and child processes, not Minecraft artifacts; examples intentionally deny because no immutable server assets are acquired. | `A-NETWORK` |
| Bounded observability probes and source mutations | [verification](../operations/verification.md) | `scripts/check-observability.py`; `crates/lkjmc-daemon/src/tests/observability_correlation.rs`; `crates/lkjmc-daemon/src/tests/observability_support.rs` | `scripts/check-observability.py` | none | Database-backed correlation and bundle probes require PostgreSQL; all events state their local source and do not satisfy independent attestation. | `B-O` |

## Boundary

No result is inferred from a prior run. Each claim needs the command, outcome,
and redacted evidence from the run being reported.
