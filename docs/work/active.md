# Active work

## Governing objective and disposition

Campaign [`docs/campaigns/202608310029.md`](../campaigns/202608310029.md), **Close the
Packaged Recovery Matrix in a Bounded Docker Systemd Lab**, governs this checkout. It continues,
narrows, and supersedes [`docs/campaigns/202608301257.md`](../campaigns/202608301257.md) for current
execution. The predecessor's exact repaired release retrieval remains accepted historical evidence;
its Incus/LXD clone and supported-host observations remain externally blocked. Docker is a disposable
development proof boundary only and cannot satisfy that supported-host remainder.

The active outcome is one canonical test-owned Docker lab that exercises exact packaged release
update, no-op, restart, isolated restore, interruption, fencing, and recovery through real systemd,
PostgreSQL, Minecraft processes, plugin readiness, and a status-protocol client without publishing a
host port. The lab implementation, bounded systemd substrate, exact baseline/runtime inputs, and
consent-before-mutation gate are now closed; no product fixture has been created. Minecraft startup
and final live-matrix acceptance are **BLOCKED pending explicit EULA acceptance**; this is not
inferred from Docker availability.

## Reconciled checkout and policy

- Repository `lkjsxc/lkjmc` is at `/home/coder/workspace/lkjmc`, branch `main`, starting revision
  `e8207f371350308681f1cc4e157918ac70fdfca6`. `HEAD`, fetched `origin/main`, and the public default
  branch agree (`0` ahead, `0` behind); upstream is `origin/main`. The fetch/push remote is
  `https://github.com/lkjsxc/lkjmc`. `main` is currently unprotected and no repository ruleset was
  returned.
- Initial tracked state was unstaged and staged clean. The only untracked file was the supplied
  campaign. Relevant ignored state is bounded Python `__pycache__` output under `scripts/` and
  `tests/`. There is one worktree, no submodule, and no nested repository.
- Supplied campaign `/home/coder/workspace/lkjmc/docs/campaigns/202608310029.md` is installed unchanged
  and currently untracked, SHA-256
  `bbe608eee774defb670df3386d1521f068f039526fc6bbe864211568d3f03521`. Root
  `/home/coder/workspace/lkjmc/AGENTS.md` is tracked, unchanged, and byte-identical to the supplied
  durable policy, SHA-256 `38bfe676b1f6b964f06854a85e021634f1c7d24168b09b21ada97e51fafdc193`.
- Git 2.43.0, Rust 1.97.0, Cargo 1.97.0, Python 3.12.3, Java 21.0.12, PostgreSQL client 16.15,
  Docker client/server 29.1.3, and Docker Compose 2.40.3 are available. GitHub CLI authentication and
  workflow/artifact read access work.
- Fresh focused deployer, release-archive, release-identity, and existing lab-harness tests passed:
  `python3 -m unittest tests.test_deploy_release tests.test_release_archive
  tests.test_release_identity tests.lab.test_lab_harness` ran 41 tests with no failure. No
  objective-relevant preexisting deterministic failure is known.

## Current Docker and artifact boundary

- Docker context `default` targets the local default Unix socket with no context TLS material and no
  `DOCKER_HOST` override. The engine is rootful, not rootless: security options are AppArmor, the
  builtin seccomp profile, and private cgroup namespaces. It uses Docker `overlayfs`, systemd cgroup
  driver, cgroup v2, Linux `x86_64`, 16 CPUs, and 12 GiB engine memory. The workspace filesystem had
  about 175 GB available and the enclosing environment about 9.9 GiB available memory at preflight.
- The daemon initially had zero containers, three default/existing networks, three unrelated unused
  volumes, and seven images. No container, network, volume, or image carried an lkjmc-related label.
  Lab resources use unique `io.lkjmc.docker-release-recovery.project` labels. No unrelated object was
  reused, stopped, deleted, or reconfigured.
- Required baseline `Verify` run `33288687707`, attempt `1`, is currently completed/successful for
  exact commit `23ad8d8ef389a453f71ffb3b0a7e333ea1e4a9d4`; all three required jobs succeeded. Preferred
  release artifact ID `9725523129` remains unexpired through `2026-09-29T03:08:05Z`, size
  `23,539,404`, with service digest
  `sha256:eeed00ebd5d7dbf3263ff2afaf7b9f12b45ba7632320773bc936a18c4da5a70a`. A fresh current-campaign
  download into private root `/tmp/lkjmc-202608310029-inputs.VIvSge` reproduced that outer digest.
  Safe three-file transport extraction and the exact baseline revision's canonical verifier,
  consumer, and extractor accepted archive digest
  `7a5c98b0fc066e7f9930562e4f8d5ce71443691e097c32d9d331a5ddcf3e7df8`, manifest digest
  `ec91332d49f5ba61f991b4bd4767007e3b53e89f33f898a6aaaf1ec71c48af8c`, all fourteen artifact
  paths/modes/digests, and embedded identities. The temporary detached baseline worktree was removed.
- Exact immutable server inputs were freshly resolved from PaperMC's maintained API and downloaded
  through content-object URLs: Folia `1.21.11` stable build `14`, 55,082,693 bytes, SHA-256
  `f52c408490a0225611e67907a3ca19f7e6da2c6bc899e715d5f46844e7103c39`; Velocity
  `3.4.0-SNAPSHOT` stable build `563`, 17,816,778 bytes, SHA-256
  `fe53021f3168322cb6cb68f78699866fd098df3c306e4359847a10b0d02689ef`. The private v1 descriptor
  validates all runtime, release, and asset bytes and reports only the expected absent target and
  EULA blockers.
- Prior private operator roots `/tmp/lkjmc-202608301257-operator.BbtjLv` and
  `/tmp/lkjmc-202608300910-operator.OY8ekk` still exist as current-user-owned `0700` directories.
  They remain predecessor evidence and are not current retrieval or runtime authority.
- The pinned runtime is Ubuntu 24.04 amd64 manifest
  `sha256:1e0a86e57d247923571b75e0aaf48a1449cf8c543d51fb3e07a4a7d7bfa79316` with the
  `20260830T000000Z` Ubuntu snapshot, exact locked packages, Java 21, and PostgreSQL 16. A rootful
  non-privileged probe ran real `/usr/lib/systemd/systemd` as PID 1 with private cgroups, an internal
  network, zero host ports, zero volumes/binds/devices, all capabilities dropped then the bounded
  twelve-capability closure restored, and no seccomp override. `SYS_ADMIN` plus
  `apparmor=unconfined` are the empirically required exceptions for the test-owned entrypoint to mount
  cgroup v2 inside its private tmpfs/cgroup namespace; the Docker socket, host PID/network, and
  `privileged` mode remain absent.
- Systemd probe evidence is private at
  `/tmp/lkjmc-202608310029-docker.8Rh7w7/systemd-probe-19.json`, SHA-256
  `77f26033d767c9e5c061d6033b586b9713b6e320155a1b7c57d210edd818b1e4`. It observed the real probe
  unit and cgroup, graceful systemd shutdown, and Docker restart replacing container start metadata,
  host PID, PID 1 start ticks, and unit-process start ticks. Cleanup removed the exact container,
  internal network, and temporary image; project-label enumeration was empty for containers,
  networks, volumes, and images afterward.
- Canonical test-only entrypoint `scripts/run-docker-release-recovery-lab.py` and the single v1 input
  contract now own exact workflow-artifact retrieval, current canonical consumption, PaperMC stable
  content-object freezing, real-systemd image construction, fresh baseline fixture creation,
  Docker rollback-point capture, wrong-manifest/no-op fingerprints, changed update, service/container
  restart, independent updater-backup verification, isolated fresh restore and target-daemon boot,
  external prepared-fence interruption, fenced restart, packaged recovery, changed-ledger refusal,
  bounded indexed evidence, secret scanning, and exact-label cleanup. Lab support remains outside
  `config/release-artifacts.json` and runtime hosts contain no checkout, Cargo, Gradle, Git, or
  mutable resolver.
- Current-image missing-consent evidence at
  `/tmp/lkjmc-202608310029-docker.8Rh7w7/fixture-consent-gate-2.json`, SHA-256
  `1e1ef7c269303c19436302698bf57517910c6d477f3797687771fc595dfe23d7`, records outer `PASS`, inner
  fixture `BLOCKED` with exact reason `explicit Minecraft EULA acceptance is absent`, and a no-effect
  oracle covering service identity, PostgreSQL cluster, canonical roots, and EULA marker. Its
  self-excluded index SHA-256 is
  `6469d3cc8bb3fd687e09c0c23add0f2f61f5645413cefcd3a09b55bad9e05a85`; canonical secret scan and
  exact-label cleanup passed with no residual container, network, volume, or image.
- The canonical preparer freshly reproduced the retained baseline and assets in private root
  `/tmp/lkjmc-202608310029-canonical-inputs.C18R62`. Preparation evidence SHA-256 is
  `d6249054bad6b4d4e11e80309602d0f1a0c755e9c8ede4f92fc85045b00ebc97`; its exact indexed v1
  descriptor SHA-256 is `f8c2d1456f2c28b039b2659b74287cd787f8ad805bd9f5b5945842392d383825`.
  It binds the Dockerfile, Compose file, all image build inputs, package lock, baseline, and both
  server assets. Its target is deliberately null and its consent value false.
- The first automated preparation attempt invalidated the assumption that an older retained release
  could be consumed directly by the dirty active checkout: the canonical archive owner correctly
  binds verification to one clean exact source commit. The driver now creates a private detached
  worktree for the artifact's exact commit, runs that revision's canonical verifier, consumer, and
  extractor, and removes the worktree. A second fresh run passed; `git worktree list` again contains
  only the active checkout.
- Fresh focused lab/deployer/archive/identity/existing-harness tests ran 55 tests with no failure;
  bootstrap, asset, and full operations mutation/contract checks also passed. The lab tests include
  the credential-scan falsifier and are now part of `verify-full.sh`.
- A fresh host-local `./scripts/verify-full.sh` passed after the final implementation change. Its
  database-backed rows were explicitly skipped because no database URL was supplied; the required
  PostgreSQL execution remains assigned to the fresh Compose and remote workflow lanes.
- No final implementation commit or target workflow artifact exists yet.
  `LKJMC_ACCEPT_MINECRAFT_EULA=1` was not supplied. No Minecraft server, PostgreSQL fixture,
  listener, player, Incus/LXD target, or production state was accessed or mutated.

## Evidence state and next executable action

**SOURCE INSPECTED**, **IMPLEMENTED** (complete lab path), **UNIT TESTED**, current
**RELEASE ARTIFACT VERIFIED** (baseline), **PROCESS TESTED** (real systemd probe), and
**DISPOSABLE DOCKER NETWORK OBSERVED** are current at their stated boundaries. PostgreSQL fixture
proof, packaged matrix rows, target release verification, and protocol-client observation are
**NOT RUN**. Fresh supported-host installation and Incus/LXD restart are **DEFERRED/BLOCKED**;
real-player and production observation are **NOT RUN**.

Next: run fresh repository verification, create and push the coherent implementation commit, require
its green `Verify` workflow, and prepare the exact retained target input. Do not write an EULA marker
or start Minecraft until explicit operator consent exists.
