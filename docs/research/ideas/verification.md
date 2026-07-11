# Verification research ideas

## Purpose

Imported falsification candidates; none replaces required real integrations.

## Catalog evidence

Source: supplied `experiments/catalog/verification.md`. Each remains untested.

## Candidates

- `QV-PROPERTY` pure properties; `QV-STATE-MODEL` transition models.
- `QV-FAULT-HOOK` deterministic failpoints; `QV-FUZZ-JSON` parser fuzzing.
- `QV-LOAD-HTTP` PostgreSQL transport load; `QV-LOAD-PLUGIN` Java-client traffic.
- `QV-PROTOCOL-BOT` real Minecraft journeys; `QV-MUTATE-CHECKS` guard mutation.
- `QV-MIGRATION-MATRIX` schema/cutover tests; `QV-RESTORE-PROOF` restored boot.
- `QV-CLOCK` deterministic time; `QV-RANDOM-SEED` retained minimization seeds.
- `QV-FLAKE-BAN` reject hidden retries; `QV-COVERAGE-MAP` state-to-proof mapping.
- `QV-SECRET-CANARY` output canaries; `QV-PERF-BUDGET` regression budgets.

## Decision boundary

Apply state models and failpoints to selected workflows, mutation-test new guards,
and run load with faults. Real client, cluster, and external-service access must
name exact prerequisites and cannot be reported as passed when skipped.
