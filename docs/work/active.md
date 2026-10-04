# Player-first release candidate

Implementation base: `3e7af86a54902188898bcc04ef761f81a7f8a28a`.
Integration branch: `work/player-first-20261004`.

The accepted direction is implemented across Core, Web, Paper, Velocity, and the
host agent: Play, Worlds, People, Timeline, separate Hosting, committed temporary
End Expeditions, and complete separately rendered English/Japanese system text.
Existing accounts, worlds, ownership, balances, and physical receipts are retained.

## Changes verified during integration

- PostgreSQL checks cover private presence and resume selection, conversation
  visibility, participant-scoped journal pagination, immutable Expedition admission,
  typed machine/game state, concurrent reads, restore exclusion, console uncertainty,
  lease fencing, and economic/identity invariants.
- Single-connection server-detail reads pass without nested pool acquisition.
  Missing, stale, and future observations never present a world as ready.
- Provenance migration checks preserve custom names and asset/achievement text.
  Rust, TypeScript, and Java message contracts reject malformed parameters and
  prevent raw diagnostics from leaking into a selected-language interface.
- Real Core/Chromium journeys passed for navigation, conversation/message/report
  persistence, and a mobile land form. Controlled browser fixtures separately
  cover private read boundaries, dialogs, locale changes, and runtime UI states.
- Actual development Paper passed 15 Expedition checks: five forced JVM crashes,
  session and combat gates, committed membership after party changes, exact costs,
  refunds once, inventory preservation, return/re-entry, latest-entry origin,
  offline expiry, safe respawn fallback, and owned deletion.

The crash suite exposed previously unsaved native world identity metadata. Paper
now flushes and fsyncs that metadata before publishing world readiness. The failed
fixture and its database were preserved; a new isolated fixture passed without
rewriting registered identities or bypassing mismatch checks.

## Release and production evidence

The complete committed candidate must pass the isolated `ops/ci/check.py` gate;
its private acceptance receipt and exact artifact manifest are the release proof.
Integration results do not substitute for that final gate.

A fresh authenticated read at 2026-10-04 15:51 UTC verified the canonical host/root
and Incus pool identities, clean GitOps `67d7e188`, and running application
`3e7af86`. There were zero active sessions, unfinished jobs, reported players,
active Expeditions, and stale server observations. This is a timestamped preflight,
not deployment approval evidence for a later state.

Deployment uses protected source/CI/GitOps publication, a fresh admission-gate
check, verified recoverable database/world backups, and the exact approved artifact.
No source record may claim deployment from a local test result. Licensed Java,
real Bedrock/console devices, voice, external WAN, and full production recovery
remain separate acceptance boundaries described in `docs/acceptance.md`.
