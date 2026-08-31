# Active work

## Governing objective

Campaign [`docs/campaigns/202608312120.md`](../campaigns/202608312120.md), **Rust-Owned Fresh
Supported-Container Installation and Operator Acceptance**, governs this checkout. Its immediate
predecessor [`202608311119.md`](../campaigns/202608311119.md) is complete through source,
deterministic, PostgreSQL/process, retained-release, and independent-retrieval evidence only. It
did not install or run a supported target. The earlier fixed-topology campaign is historical only
for recovery scenario semantics.

The active outcome is a UUID-bound Rust `lkjmc-ops host install` operation for a prepared systemd
container. It anchors a release, configuration, and immutable server assets; creates only a fresh
owned identity/root/database closure; publishes the exact service; and distinguishes accepted
replay, resumable work, and conflicts. Supported-host, player, and production evidence remain
separate external boundaries.

## Exact repository and release identity

- Repository `/home/coder/workspace/lkjmc` (`lkjsxc/lkjmc`), branch `main`, remote
  `https://github.com/lkjsxc/lkjmc`. The exact behavior and release implementation commit is
  `daca26877d7923734e04b685f3db6f5ea85646f8`, pushed to `origin/main`. No behavior-affecting source
  or release-inventory change follows it; this ledger update is evidence-only.
- Supplied durable policy SHA-256: `96a3188351b086769f533d8de4d75bfcd596401a3fff98c4f627e0f625382299`
  (`AGENTS.md`). Governing campaign SHA-256:
  `c61f63db77c3f6817f63fa3cf94c18785db29db9e9b373715371618989d6fa2e`.
- The predecessor’s GitHub Verify run `33373201044` for
  `13602f277bcc109d891bc3726976a0323936e8b3` succeeded. Its retained archive and consumer receipt
  were independently retrieved; that remains **RELEASE ARTIFACT VERIFIED**, not installation or
  player evidence.
- Final implementation Verify run `33406882844` succeeded: docs contracts, clean Compose gate,
  two fresh release roots and archive comparison, secret scan, canonical artifact upload, and its
  separate consumer job. Artifact `9764670888` remains retained through 2026-09-30. Its outer
  artifact-service digest is
  `sha256:823333b3b23acc45209dbee7af34dc9ced5639b0d45d9942eaac59327ed37d2e`; inner archive SHA-256
  is `9efe2b98645c403fb117658a64bd9e2de6b01c62a47be8c5a16cf842205282bb`; release-manifest SHA-256
  is `4a74f245e4481ecf5dbf381aa2dc69e6632a9442adf72a3ceb3951d349bfc44f`.
- The final artifact was independently downloaded after retention, checked against the outer
  digest, passed the canonical archive verifier and an offline pinned-image consumer without a
  rebuild: eight payload artifacts and 110 contracts bind to `daca2687…`. The downloaded archive
  contains no `lkjmc-discord` path. This is **RELEASE ARTIFACT VERIFIED**, not an installed release.

## Implemented and verified slice

- `42d0e931` retired the disabled-only `lkjmc-discord` executable and its direct
  release/build/test/current-document consumers. The independent inventory is three Rust binaries,
  three Java jars, and two declarative systemd files; source and final-archive residual searches
  are clean outside historical context.
- `lkjmc-ops host install --input PATH --input-sha256 DIGEST` has one strict versioned Rust input
  owner and a secret-free durable journal. It preflights systemd, Java, PostgreSQL 14, tools,
  capacity, listener availability, identity/path/database conflicts, release/configuration/asset
  anchors, and topology-neutral typed configuration before lkjmc mutation.
- It creates the dedicated `lkjmc` identity and private roots, scoped atomically published secrets,
  UUID-marked local PostgreSQL role/database/migrations, exact release/assets/plugins, root-owned
  EULA policy/materialization, systemd unit/drop-in, fence, and one-use permit. Acceptance rechecks
  persisted fleet equality, daemon/Velocity readiness, systemd cgroup state, and exact daemon
  executable/UID/start identity before clearing the fence. Exact accepted replay is read-only.
- The daemon honors its configured runtime-asset root. Altered EULA policy/files and non-exact
  existing releases fail closed rather than being rewritten. A stale Cargo-package expectation was
  corrected from 215 to 214 after Discord retirement; it did not change the intended eight-member
  payload.
- **FORMATTED**, **STATICALLY CHECKED**, **UNIT TESTED**, **POSTGRESQL TESTED**, and
  **GENERATED-CONTRACT VERIFIED** apply to the focused pinned checks. The remote clean Compose gate
  and two-root release closure provide fresh broader deterministic/release proof for the exact
  implementation commit.

## Current blockers and limits

- A prior clean detached-worktree local `verify-full` run passed its initial Rust, PostgreSQL,
  process, network, observability, runtime, sync, fault, and daemon/CLI stages, then became
  **BLOCKED** in its final local JVM/check phase when Docker metadata exhausted its fixed disk
  quota. It was not a source assertion failure; the successful remote exact-run result is separate
  stronger release evidence.
- Docker is proof infrastructure only, not supported-host evidence. The campaign-owned local
  containers were removed, but Docker retained stale endpoints on one disposable verifier network
  after the quota event. Do not restart the shared daemon or run a broad prune to force removal.
  No lkjmc service, database, system container, or player listener was created by that local lane.
- Read-only discovery found no `incus` or `lxc` client and no active Incus/LXD service on the
  authorized checkout host. Consequently **FRESH SUPPORTED-HOST INSTALLED**, **OPERATOR OBSERVED**,
  **PROTOCOL-CLIENT OBSERVED**, service/container restart, backup, and isolated-restore evidence
  are **BLOCKED** by the absent authorized manager. Docker must not be substituted.
- **REAL-PLAYER OBSERVED** and **PRODUCTION OBSERVED** are **NOT RUN**. No public traffic,
  production, or ambiguous external target was changed.

## Next executable action

When an authorized host exposes one authoritative Incus or LXD manager and a project-owned fresh
unprivileged system-container boundary, prepare the declared substrate plus an exact installation
input and immutable runtime-asset closure for the retained `daca2687…` release. Run the packaged
first installer there, then perform the campaign’s operator, protocol, no-op, restart, backup, and
isolated-restore acceptance. Until then, preserve this explicit blocked state without using Docker
as a substitute.
