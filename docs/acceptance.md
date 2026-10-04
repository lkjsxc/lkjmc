# Acceptance

Implementation, fixture coverage, actual adapter execution, and public production
acceptance are separate claims. A successful build or HTTP response does not prove
that a real player completed a journey.

The release gate in `ops/ci/check.py` runs an isolated, offline, non-root image
with PostgreSQL 18, fixed dependencies, source/history secret classification,
Rust and guest tests, a reproducible release build and archive verification,
web state tests, browser fixtures, real Core/browser integration, and actual
Paper/Velocity protocol scenarios. The protocol lane also exercises Expedition
entry/return and crash recovery. Its Java clients use offline development identities.

| Area | Required evidence | Scope of the automated gate |
|---|---|---|
| Player access | Authorized resume, private presence, bilateral blocks | PostgreSQL and browser contracts |
| Interface | Mobile/desktop navigation, readable actions, open-dialog language changes | Real Chromium fixtures and Core-backed browser journeys |
| Conversations | Selected-room reading and sending, membership loss, deleted messages | PostgreSQL, API/state, and browser tests |
| Expeditions | Committed roster, payment, return/re-entry, disconnects, durable crash recovery | PostgreSQL plus actual development Paper |
| Runtime | Honest console uncertainty, responsive authorized reads, effect fencing | PostgreSQL, mock host boundary, real guest-helper filesystem tests |
| Backups | A dump restored into an independent database with retained balances/ledger | Actual PostgreSQL dump/restore; not a full Incus/world restore |
| Release | Exact commit, fixed build inputs, verified archive | Isolated CI acceptance receipt and release manifest |

The following require separate evidence and must never be inferred from the
local gate: licensed Java authentication; real Bedrock/Geyser/Microsoft linking;
console devices; external WAN TCP/UDP ingress; live voice calls; real Incus wake,
restore and power-loss recovery; production browser behavior; sustained load.

Before deployment, verify fresh canonical host identity, global sessions and
unfinished work, drain Expeditions and transfers through the existing admission
gate, and independently verify recoverable database/world state. Deploy the exact
CI-approved artifact through GitOps and retain the saved-plan and deployment
receipts. Never overwrite player data to satisfy a check.

Current candidate outcomes and remaining access requirements are recorded in
[the active work record](work/active.md). Runtime acceptance receipts remain in
their private task/CI evidence directories; no credentials or database dumps are
committed here.
