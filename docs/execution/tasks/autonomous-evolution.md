# Autonomous evolution

## Purpose

This ledger records the active documentation-first evolution of `lkjmc`.
Repository owner documents and executable evidence remain authoritative.

## Status

active

## Controller

The temporary controller at
`tmp/lkjmc-autonomous-evolution-plan/control/planctl.py` owns task state. Its
terminal command is the only completion permission. This committed ledger records
truthful repository evidence and never replaces controller state.

## Baseline

- `./scripts/check-lines.py`: passes from a clean generated-artifact tree.
- `./scripts/check-docs.py`: passes.
- `./scripts/verify-fast.sh`: passes with declared skips.
- `./scripts/verify-full.sh`: passes with live skips.
- `docker compose --profile verify run --rm verify`: passes with live skips.
- `./scripts/verify-live.sh`: reports `ran=none`; every live lane is skipped.

The baseline also proved that nested Gradle output makes `check-lines.py` fail
because the checker does not exclude `platforms/**/build`. This is an internal
verification defect, not a passing check.

## Prior acceptance reconciliation

The previous ignored plan was recovered from a local archive and compared against
current source, tests, and current contracts. Its 66 acceptance items map to the
new graph; none is retained merely because a narrower field, check, runbook, or
skip exists.

- 42 items are reopened.
- 21 items require a stronger replacement.
- 3 live items remain external-proof-pending.

The retained forensic evidence is under
`tmp/lkjmc-autonomous-evolution-plan/.control/artifacts/O-FORENSIC/`.

## Reopened internal work

- Generic command payload schemas and incomplete cross-surface bindings.
- Daemon-wide runtime serialization and missing durable operation/reconcile facts.
- Authorization, scoped credential, web attribution, economy, and profile safety.
- Menu schema, route, stale-state, scheduler, localization, and transfer proof.
- Kubernetes provisioning truth, migration integrity, atomic downloads, restore,
  toolchain provenance, and truthful verification output.

The complete mapping is controller evidence. The active graph prevents these
findings from being silently closed by prose.

## Documentation barrier

`D-INVENTORY` began the documentation campaign. Before `DOC-GATE`, product and
runtime behavior remains frozen. The independently approved `D-DOC-CHECK` and
`D-DOC-CHECK-HARDEN` tasks are the sole verification-only exception: they changed
only documentation checker paths to enforce the proof gate.

## External proof prerequisites

- Minecraft: `LKJMC_MINECRAFT_SMOKE=1` and reachable runtime prerequisites.
- Minecraft claim: `LKJMC_MINECRAFT_CLAIM_SMOKE=1` and a test database/runtime.
- Playable: `LKJMC_PLAYABLE_SMOKE=1` and explicit Minecraft EULA acceptance.
- Bedrock: `LKJMC_BEDROCK_SMOKE=1` plus a supported endpoint and client.
- Discord: `LKJMC_DISCORD_SMOKE=1` plus real credentials and interaction access.
- Kubernetes: `LKJMC_KUBERNETES_SMOKE=1`, `kubectl`, and an authorized disposable
  namespace.

A missing prerequisite is recorded only after its guarded command is attempted.

## Next executable step

`D-VERIFY` independently accepted the complete documentation proof. `DOC-GATE`
now performs the final clean-worktree integration check; only the controller can
unlock the first foundation implementation task.
