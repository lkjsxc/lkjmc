# Active work

## Governing objective and disposition

Campaign [`docs/campaigns/202608311119.md`](../campaigns/202608311119.md), **Rust-Owned
Topology-Neutral Update and Recovery Cutover**, governs this checkout. Campaign
[`docs/campaigns/202608310029.md`](../campaigns/202608310029.md) is implemented in source but
externally incomplete and is superseded for current execution. Its fixed-topology Docker matrix was
not run against its exact release; its scenario semantics remain historical input until the Rust
tests own them, after which its obsolete lab owners are to be deleted.

The active objective is one packaged `lkjmc-ops` Rust authority for release verification, artifact
publication, update/no-op/recovery, PostgreSQL backup/restore verification, fencing, EULA
materialization, post-start reconciliation, and diagnosis over a bounded typed fleet. The cutover
must remove shipped Python/shell operational executables and all `proxy`/`hub`/`survival` name or
backend-count semantics from that boundary.

## Reconciled checkout and policy

- Repository `lkjsxc/lkjmc` is at `/home/coder/workspace/lkjmc`, branch `main`, starting revision
  `8d55e156dad7c22493b7c9c35b6520ae9a271fa0`. Fetched `origin/main`, local `HEAD`, and the public
  default branch agree (`0` ahead, `0` behind); upstream is `origin/main` and the fetch/push remote is
  `https://github.com/lkjsxc/lkjmc`.
- GitHub's public branch response reports `main` unprotected and no repository ruleset. Starting
  revision workflow `Verify`, run `33327513627`, attempt `1`, is completed/successful as of
  `2026-08-30T18:38:26Z`. GitHub CLI is unavailable, so authenticated push capability is not yet
  established.
- Initial index and tracked product state were clean. The supplied replacement `AGENTS.md` was the
  only unstaged tracked change, and the supplied campaign was the only untracked path. Relevant
  ignored state is bounded Python `__pycache__` output under `scripts/` and `tests/`. There is one
  worktree, no submodule, no nested repository, and no subtree `AGENTS.md`.
- Root `/home/coder/workspace/lkjmc/AGENTS.md` is installed unchanged from the supplied policy,
  SHA-256 `370e62433444a22bc350feac5fda516c7e7a419cd0e36e20d8b9c04792c6fcbf`; its committed predecessor
  hash was `38bfe676b1f6b964f06854a85e021634f1c7d24168b09b21ada97e51fafdc193`.
  `/home/coder/workspace/lkjmc/docs/campaigns/202608311119.md` is installed unchanged, SHA-256
  `21a1a1af2a1854a602eb3bb26c6dfdf2150d02dc6c6ada7a7d5ec58d3af36161`.
- Git 2.43.0, Python 3.12.3, systemd 255, and Docker client/server 29.1.3 are available. Rust 1.97.0
  and Cargo 1.97.0 are installed under `/home/coder/.cargo/bin` but not on the inherited `PATH`.
  Java, Gradle's Java runtime, PostgreSQL client tools, and GitHub CLI are unavailable on the host;
  Docker remains available for isolated final tiers only after deterministic prerequisites.
- A fresh predecessor-focused run of 52 tests had 49 passing assertions and three environmental
  failures: two missing `cargo`/`javac` executables and one Gradle assertion masked by missing Java.
  No behavior failure was observed. Future commands must set the Rust path explicitly; Java and
  PostgreSQL proof require a pinned container or installed toolchain.

## Objective-critical source facts

- `config/release-artifacts.json` schema v1 currently inventories fourteen artifacts. Six shipped
  operational executables are interpreter-owned: `lkjmc-deploy-release`,
  `lkjmc-install-artifacts`, `lkjmc-backup-postgres`, `lkjmc-restore-postgres`,
  `lkjmc-bootstrap-after-start`, and `lkjmc-deployment-fence-check`. The canonical systemd unit
  directly invokes the last two. The deployer invokes the other operational programs and owns the
  fixed installed-layout expectations.
- Maintained predecessor consumers are the release inventory, systemd unit/drop-in, deployer tests,
  release/operations checks, update/restore drills, the Docker recovery lab, release and operations
  documentation, and workflow verification. `scripts/install-artifacts.sh` is an additional wrapper
  with no independent product authority.
- `lkjmc-bootstrap-after-start` hardcodes `hub` and `survival`, parses the EULA policy in shell,
  invokes Python to interpret bootstrap output, and passes a request-scoped EULA flag. The deployer
  also embeds fixed topology, ports, plugins, credential paths, EULA checks, and readiness checks.
- `lkjmc-core` is the canonical typed owner for `LkjmcConfig`, `InstanceFileConfig`, `InstanceId`,
  `InstanceKind`, `DesiredState`, and `ObservedState`. Instance inventory is persisted generically by
  ID in PostgreSQL. Current config does not yet carry an explicit readiness/integration contract, so
  that typed boundary must be added rather than independently reimplemented in operations code.
- Rust `InstanceKind` includes `purpur`; migration `002-instances.sql` does not. Rust
  `DesiredState` includes `suspended`; the database constraint does not. Observed-state alignment was
  expanded by migration 036 and currently agrees. A new migration is required; released migrations
  remain immutable.
- Request-scoped EULA authority remains in CLI bootstrap parsing, daemon bootstrap/instance/
  temporary/adventure/shop handlers, generated command contracts, templates, Compose/dev helpers,
  tests, and docs. No distinct maintained legal-consent consumer has yet been proven. The canonical
  host marker path is `/etc/lkjmc/minecraft-eula.accepted`, but its parser/materializer must move to
  Rust and exact per-instance `eula.txt` targets must be inventory-derived.
- Private historical predecessor release bytes and capacity evidence still exist in owner-only roots.
  They remain historical evidence, not a target release for this campaign. No installed service,
  PostgreSQL database, Minecraft process, public listener, supported host, player, or production
  environment was observed or mutated during reconciliation.

## Completed dependency slices

- Governing policy, immutable campaign, and reconciled ledger were committed as
  `c3c4f298c122499142622932ced3474f80e3911a` (`docs: govern Rust operations cutover`). No historical
  campaign was edited.
- The new workspace crate `lkjmc-ops` now directly owns anchored release verification, exact
  installed-tree publication/no-op checks, trusted bounded subprocesses, typed fleet comparison,
  root-policy EULA materialization, a global lock, durable journal/fence and one-use permit,
  PostgreSQL backup/restore verification primitives, dynamic post-start status verification, and
  bounded diagnosis. It does not execute a predecessor script.
- Canonical `lkjmc-core` network inputs now carry typed integration and readiness contracts. A new
  migration `054-align-instance-kind-and-desired-state.sql` aligns PostgreSQL instance-kind and
  desired-state constraints without editing released migrations.
- Rust tests own the first preserved behavior set, including exact release identity, no-op and
  interrupted publication, lock conflict, fence/permit replay resistance, migration-ledger recovery
  classification, backup metadata rules, dynamic inventory comparison, status-protocol probing,
  EULA policy/materialization, and both required noncanonical fleets.
- `PATH=/home/coder/.cargo/bin:$PATH cargo fmt --all -- --check`, focused Clippy with `-D warnings`,
  and `cargo test -p lkjmc-core -p lkjmc-ops` pass. The latest focused run executed 69 core tests,
  20 operations-library tests, and one operations-CLI test.

## Evidence state and next executable action

Current state is **IMPLEMENTED AND UNIT TESTED** for the typed inputs and first Rust operational
primitives. Starting remote workflow is **OBSERVED SUCCESSFUL**. Real PostgreSQL, complete changed
update/recovery, systemd/process integration, generated artifact, final release artifact,
installation, operator, protocol-client, real-player, supported-host, and production proof for this
campaign are **NOT RUN**. The predecessor Docker capacity blocker is historical and does not block
deterministic cutover.

Next: complete and fault-test the direct Rust changed-update and interrupted-recovery state machine,
including exact no-op, backup-before-fence, migration-ledger classification, safe pre-ledger
rollback, data-aware restore requirement, and independent post-start acceptance. Then cut systemd
and the release inventory over once before deleting predecessor authorities.
