# Active work

## Governing objective

Campaign [`docs/campaigns/202608311119.md`](../campaigns/202608311119.md), **Rust-Owned
Topology-Neutral Update and Recovery Cutover**, governs this checkout. Campaign
[`docs/campaigns/202608310029.md`](../campaigns/202608310029.md) is implemented in source but
externally incomplete and superseded for current execution. Its fixed-topology Docker target was not
run; the useful no-op, failure, fence, rollback, and recovery semantics are now Rust-owned tests.

The active objective is one packaged `lkjmc-ops` Rust authority for release verification,
publication, update, recovery, PostgreSQL backup/restore verification, EULA materialization, fencing,
post-start verification, and diagnosis over a bounded typed fleet.

## Checkout and policy identity

- Repository `lkjsxc/lkjmc` is at `/home/coder/workspace/lkjmc` on `main`, upstream
  `origin/main`. Work began at `8d55e156dad7c22493b7c9c35b6520ae9a271fa0`; `origin/main` and the
  starting checkout were equal. The accepted dependency-closed cutover checkpoint is
  `64d69a76d1e1fb2dcbd2fdf8453a949e7edfd8da`; the exact locally built release revision is the clean
  evidence commit `92d036090bcd2cb1c8a27fd8f77afd08deeae093`. This final ledger update follows that
  release revision and changes evidence prose only. The initial product tree was clean and all
  campaign changes were committed.
- There is one worktree, no submodule, nested repository, or subtree instruction file. Relevant
  ignored state consists of Cargo/Gradle build output and bounded test `__pycache__` output; none is
  release authority. The final verifier used an empty agent-owned Cargo target bind-mounted over the
  checkout target rather than consuming ambient output.
- Root `AGENTS.md` is installed at SHA-256
  `370e62433444a22bc350feac5fda516c7e7a419cd0e36e20d8b9c04792c6fcbf`. The immutable governing
  campaign is installed at SHA-256
  `21a1a1af2a1854a602eb3bb26c6dfdf2150d02dc6c6ada7a7d5ec58d3af36161`.
- Starting public workflow `Verify` run `33327513627` concluded successfully for the starting
  revision. Public metadata reported no branch protection or repository ruleset; authenticated push
  capability and the exact final workflow remain to be established.
- Host Git, systemd, Docker, Python, and an out-of-`PATH` Rust toolchain are available. Host Java,
  Gradle runtime, PostgreSQL clients, and GitHub CLI are absent. Verification uses the pinned local
  image `lkjmc-rust-cutover-verify:local`; real PostgreSQL proof uses a project-scoped PostgreSQL 14
  container with no published port. No shared Docker state was pruned or reconfigured.

## Implemented cutover

- `lkjmc-ops` directly owns strict anchored release parsing, exact installed inventory and no-op,
  root/system install, durable deployment lock/journal/fence/one-use permit, changed update,
  interruption recovery, safe pre-ledger rollback, data-aware recovery classification, PostgreSQL
  backup and isolated restore verification, canonical EULA policy/materialization, dynamic
  post-start verification, and bounded diagnosis. It invokes no predecessor interpreter program.
- A preflight journal is fsynced before EULA, rollback-input, or backup effects. Pre-fence recovery
  verifies the unchanged source and records `abandoned`; it removes only an exact owned UUID-bound
  partial backup stage. Verified final backups remain journal-bound.
- The release inventory is nine members: four native Rust binaries, three Java jars, and two
  declarative systemd files. The unit and fence drop-in invoke only `lkjmc-ops`. The six shipped
  interpreter authorities, their wrappers, sole-consumer drills/tests, and the obsolete
  fixed-topology Docker recovery lab are deleted.
- Canonical configuration carries typed integration and readiness. Fleet, plugin, credential, EULA,
  listener, route, status, and persisted-inventory decisions iterate stable instance IDs. Generated
  heartbeat credential paths derive from configured data state. IPv4 and IPv6 listener formatting is
  unambiguous. Velocity reports its observed registration set through authenticated heartbeat data;
  no daemon command fabricates that observation.
- Migration `054-align-instance-kind-and-desired-state.sql` aligns retained Rust enum values and
  PostgreSQL constraints without changing released migrations. Request-scoped EULA booleans and
  confirmation handlers are removed. Creation persists typed state; start remains fail-closed on the
  root-owned host policy and exact per-instance EULA file.
- Two noncanonical inventories cover one arbitrary backend and three differently named Paper/Folia/
  Purpur backends, including an intentionally stopped backend, dynamic target enumeration, persisted
  equality, readiness, exact no-op, and named drift rejection.

## Current evidence

- **Formatted and statically checked:** `cargo fmt --all -- --check` and workspace Clippy with
  `-D warnings` pass in the pinned verifier after the Rust cutover. Contract generation/validation
  reports 132 commands; configuration examples parse through Rust; bootstrap-document checks pass.
- **Unit/process tested:** all 37 `lkjmc-ops` library tests and its CLI parser test pass. They include
  anchored identity, atomic publication faults, first-cause persistence, lock conflict, fence/permit
  replay, recovery classification, EULA symlink rejection, both noncanonical inventories, bounded
  subprocess output, and process-group timeout with independent descendant-death observation.
- **Integration tested:** the noncanonical daemon network process matrix passes all 10 apply,
  reapply, drift, interruption, restart, and concurrency scenarios. The Rust-generated EULA state is
  consumed by daemon start checks, and the exact group-readable `0640` contract is observed without
  being rewritten. Targeted daemon credential, status, adoption, and pool-release tests pass.
- **PostgreSQL tested:** fresh migration enum/constraint alignment, version-53 upgrade, migration
  checksum/lock/deadline probes, inventory/status heartbeat writes, deployment state reads, and the
  real `pg_dump`/`pg_restore --list`/fresh-target restore/corruption-rejection test pass against an
  isolated PostgreSQL 14 container.
- **Java tested:** affected common, Velocity, and Paper Gradle tests pass; generated JVM command
  bindings were regenerated and checked after command deletion.
- **Full deterministic/PostgreSQL/process gate:** `./scripts/verify-full.sh` passes in the pinned
  verifier with PostgreSQL, claim, and web probes enabled. Its final receipt reports every required
  data-workflow, network-adoption, runtime-adoption, sync-adoption, database-isolation, security,
  fault, safe-operations, daemon/CLI, process, jar-registry, claim, web, and JVM gate as run. The only
  explicit skips are nested-Docker secret-context inspection (Docker is not exposed inside the
  verifier) and the opt-in jar-live/plugin-assets lanes. They are not acceptance evidence.
- Environmental attempts before the accepted full run are retained as failures, not product passes:
  one moved Cargo cache selected a host-GLIBC binary; one clean retry exhausted the shared container
  metadata quota; and the resulting stale project-network endpoint made PostgreSQL unavailable. The
  obsolete 4.6 GB agent-owned cache and exact failed verifier container were deleted. PostgreSQL was
  recovered on a replacement project-only network using the same pinned image and disposable volume,
  with no published port. No shared image, volume, daemon, data root, or unrelated network was pruned
  or reconfigured.
- **Release artifact verified:** `scripts/build-release.sh` built clean revision
  `92d036090bcd2cb1c8a27fd8f77afd08deeae093` twice from separate detached worktrees in the pinned
  verifier. `compare-release-roots.py` reports identical roots with 13 entries and 11 files. Each
  manifest binds nine payload artifacts and 112 contract files. Strict independent extraction
  reproduces the release root; the verifier image independently accepts all four native and three jar
  build identities. ELF headers and dynamic dependencies show native binaries with only standard
  system libraries, and both systemd members invoke `lkjmc-ops` without an interpreter.
- The private local release evidence root is `/tmp/lkjmc-release-output.yNthQn`. Manifest SHA-256 is
  `a5c42ac20a9e5b58559f774d3a5ddb0d14c84c2f3101ffdad325ef2f9723f9bf`; the reproducible local archive
  is 27,603,968 bytes with SHA-256
  `ce85cc44af9c5d4225009be9b4664b9162b36d4ecf815f94b79fbb8142bb5eaa`. Its run ID is deliberately
  synthetic local metadata: it has no artifact-service ID, retention promise, or expiry and is not a
  published or independently retrieved GitHub artifact. A host identity attempt was not run because
  Java is absent; both build-time checks and the separate verifier-container check passed.

## Unobserved boundaries and next action

No GitHub artifact has been retained or independently retrieved, and no release has been installed.
No real systemd service, Minecraft/Velocity runtime, listener, disposable network, clean supported
host, operator session, protocol client, real player, public network, or production environment has
been observed. Historical predecessor artifacts and capacity measurements were not promoted.
Remaining non-shipped Python/shell build and verification owners are deferred language debt and are
not in release bytes or the runtime path.

Public branch metadata reports `main` unprotected with no repository ruleset, and remote `main`
remains the starting revision. A fast-forward push dry run is blocked before policy evaluation because
the configured HTTPS credential helper calls missing `/usr/bin/gh`; no `GH_TOKEN`, `GITHUB_TOKEN`,
askpass helper, or SSH key is available. The exact disposable PostgreSQL container, volume, healthy
replacement network, verifier caches, and release scratch were deleted. Docker refuses to delete the
original project network because quota-corrupted daemon metadata retains one endpoint for the deleted
PostgreSQL container. No live container references it; daemon restart or shared-state repair was not
attempted.

Next: provide working GitHub authentication, revalidate `origin/main`, and fast-forward push the clean
current `HEAD`. The exact push workflow must pass, build the enclosing evidence commit twice, publish
and independently consume its retained artifact, and provide the real run/artifact identities before
campaign closure. Then retry deletion of only the stale project network if the Docker daemon has
reconciled it. Optional live systemd or Minecraft proof remains outside minimum acceptance.
