# Active work

## Current objective

Recover lkjmc into the smallest deployable single-host network: one Rust daemon and CLI, PostgreSQL, Velocity, hub, survival, a real `/lkjmc` transfer path, and a small Paper/Folia menu. Status and coherent pre-release identity are committed. The current gate is pruning the generic denied/inert command surface before implementing Velocity `/lkjmc`.

## Repository state

- Inspected recovery base: `339adb7bb60b6cce5229f98866f869363e90b78b`.
- Current product commit: `34a33389adf2f594af5d3a64c02b4ce1d40488f2`; status checkpoint: `5c2d88f0b9766e5135b07927f9de982fbe1a2d84`; controller cleanup checkpoint: `db96a42e83c85def8bae34b4917d601b8d9a37ee`.
- Branch: `main`, tracking `origin/main`, four commits ahead before this ledger update.
- Worktree was clean after `34a3338`; private evidence is ignored under `tmp/agent/baseline-20260813T131635Z/`, `tmp/agent/status-slice-20260813T140037Z/`, and `tmp/agent/release-identity-20260813T150538Z/`.
- Baseline repository size: 1,373 tracked files; 390 documentation files (320 Markdown), 89 top-level scripts, 52 migration files, 453 Rust files, and 245 Java files.
- After controller/history cleanup (before commit): 1,165 projected tracked files, 189 documentation files (334,552 bytes), and 81 top-level scripts.

## Confirmed facts

### Local baseline

- Local host: Ubuntu 24.04, Rust 1.97.0, Cargo 1.97.0, OpenJDK/Javac 21.0.11, Python 3.12.3.
- The host PostgreSQL client is absent and `LKJMC_STORE_TEST_DATABASE_URL` is unset by default; no database-backed baseline lane ran. The later focused status lane used a disposable PostgreSQL container explicitly.
- `cargo fmt --check`, workspace Clippy, workspace Rust tests, the Python lab harness, and Gradle JVM tests pass.
- The inherited line checker fails only because the authorized `AGENTS.md` is 583 lines; its universal 200-line rule is obsolete.
- The inherited documentation checker rejects the new durable ledger because it requires the old per-directory README topology. This is a process-topology failure, not product evidence.
- Rust and JVM packages now share canonical version `0.1.0-alpha.1` and Apache-2.0 metadata matching the root `LICENSE`.
- Ordinary builds expose observed commit when Git is available but report dirty state as unknown. Exact release identity requires a clean matching Git checkout and fresh build nonce; gitless supplied commit claims fail.

### Current-consumer inventory

- Rust has 39 top-level CLI enum variants, 136 daemon registrations/contracts, and 130 `denied-unproved` members. Only six contracts have non-denial effects.
- The schema has 51 migration files creating roughly 87 tables for many deferred domains. Existing deployment restore verification confirmed 67 and 83 public tables in the two historical databases.
- `status` is a real Unix-socket caller and non-denial handler. At `5c2d88f` it returns one consistent, deterministic, 32-row-bounded PostgreSQL view of desired state, process observation, tri-state backend readiness, proxy registration, and joinability. Diagnostic strings are character-bounded and omissions are explicit.
- At `34a3338`, CLI, daemon status/health, daemon/Discord `--version`, JVM descriptors/manifests/constants, and JVM startup logs share one build identity. Release construction ignores ambient outputs and builds in a fresh detached worktree tied to the clean commit.
- No operator backup or database restore CLI/daemon operation exists.
- Velocity registers zero lkjmc commands. Generated Java metadata explicitly reports zero JVM command consumers.
- Paper registers `/menu` and `/docs`; its generated bundle requires 62 routes and 66 actions, while no menu mutation has a real supported effect.
- A real low-level Velocity connection request exists but is constructed and discarded; unavailable attestation prevents its current transfer adapter from truthfully completing.

### Live read-only discovery

- Authorized SSH alias `home-incus` works with an existing verified host key and a mode-0600 identity; no new host key was accepted.
- Target host is Ubuntu 26.04 with Incus client/server 7.3. Incus is authoritative; do not install another manager.
- Incus project `default` uses ZFS pool `default` and managed bridge `lxdbr0`. Unrelated containers, networks, listeners, and services are preserve-only.
- Existing unprivileged containers are `lkjmc` and `lkjmc-next`; both have `security.privileged` unset (default false), 6 CPUs, 24 GiB memory, managed NICs, and no prohibited host mount/socket observed.
- `lkjmc` runs PostgreSQL 16, the daemon, Velocity, and one hub. It has no survival backend. Its systemd working directory still points into a checkout. CLI status reports proxy/hub processes healthy but both non-joinable because plugin heartbeat/registration is absent.
- `lkjmc` exposes legacy host devices for daemon web TCP, Velocity TCP, and Bedrock UDP. They remain unchanged pending verified replacement and traffic rollback.
- `lkjmc-next` was private but its daemon had 3,038 restarts and no Java process. It is now stopped (not disabled) after preservation to bound the crash loop.
- Existing `eula=true` records were observed in both lkjmc containers; no new acceptance was fabricated.
- Existing databases are broad pre-recovery schemas: `lkjmc` restored as 67 public tables/43 migrations; `lkjmc-next` restored as 83 public tables/51 migrations.

## Preservation and rollback

- Incus snapshots created for both containers: `pre-recovery-20260813T132126Z`.
- Verified private application backup on `home-incus`: `~/backups/lkjmc/pre-recovery-20260813T132523Z/`.
- Backup contains each container's PostgreSQL custom dump, dump listing, `/opt/lkjmc`, `/etc/lkjmc`, lkjmc systemd unit, data/world roots, logs, safe metadata, Incus config/info, and checksums.
- Backup size: 589,211,407 bytes. Top manifest SHA-256: `768a121d4138aa7ecd212ab3d7d53a359a12ece2dd30270a8b44e913d384c282`.
- Verification: every file checksum passed; each filesystem archive was listed and extracted into a private disposable root; each dump restored into a new disposable database and was queried before deletion.
- The first backup attempt failed before dump creation because PostgreSQL could not traverse a root-only staging path. Failure is retained in private evidence; the corrected retry passed.
- `lkjmc` was restarted after backup and reached daemon HTTP, Velocity, and hub listener readiness in 6 seconds. `lkjmc-next` remains stopped.
- Snapshot rollback boundary (only if needed): stop the affected container, restore its named snapshot, then start it. Do not execute while the current serving deployment is healthy without recording traffic impact.

## Decisions

- Preserve the replacement `AGENTS.md` as user-owned input and use this file as the only durable active ledger.
- Make production discovery read-only except for required lkjmc-specific preservation and bounding the private crash loop.
- Do not reset either broad database until the verified backup and snapshot remain available.
- Keep Incus as the sole container manager and preserve all unrelated host resources.
- Treat the current deployed network as historical preservation evidence, not acceptance of the recovery product: it has no survival server, no plugin heartbeat, no `/lkjmc` proof, and no current player evidence.
- Remove the universal line limit and old documentation/controller topology rather than altering the replacement contract to satisfy them.
- Reuse the existing private Unix socket, status caller, local runtime ownership, PID identity fencing, and scoped TCP bearer boundary for the first status slice; do not add a second framework.
- Defer transfer and menu rewrites until status exposes a small truthful network snapshot.
- Status and `instance.list` share one typed availability decision. Future-dated, stale, missing, stopped, unhealthy, invalid-port, and proxy-only evidence fail closed with exact reasons.
- The status data read is one SQL statement; shared legacy dispatch still writes command-completion observability. This is explicitly documented and should be narrowed when generic dispatch is removed.
- Use `0.1.0-alpha.1` and Apache-2.0 as the single package identity. Ordinary builds never infer a clean state from a warm build cache; release builds require clean Git, matching commit, a fresh nonce, fresh output directories, and executable identity verification.
- Exported CI/lab source must attach a trusted Git bundle before release construction. Release output parents are protected local-build boundaries, and failure cleanup refuses a replaced output inode.

## Acceptance completed

- WP-00 baseline: complete at commit `339adb7` with exact local pass/fail/blocked classification.
- WP-01 contract/ledger: complete. Replacement contract and durable ledger are the current authority; no `planctl.py` or `DOC-GATE` reference remains in the active reading path.
- WP-02 discovery/preservation: complete. Existing deployment found; snapshots, private application backup, checksum verification, database restore verification, and filesystem extraction verification complete.
- WP-03 controller/history cleanup: complete. Deleted 201 documentation/research/controller files (about 865 KiB), seven obsolete documentation/truth checker files plus their wrapper/mapping, and the universal line checker. The executable fault replay fixture moved to `tests/fixtures/` and is now owned by the full gate.
- WP-04 consumer inventory: complete enough to select status. Counts and owner seams are recorded above; exhaustive permanent inventory is intentionally omitted.
- First status vertical slice: complete at `5c2d88f`. JSON and human CLI output were observed through a real Unix socket against disposable PostgreSQL; no new operation or framework was added.
- Release identity slice: complete at `34a3338`. A clean-commit fresh-worktree build produced six artifacts whose CLI/JVM identities, manifest, hashes, and Apache-2.0/version metadata all matched commit `34a3338`; the private disposable release was scanned and removed after bounded evidence was retained.

## Current failures and blockers

- `lkjmc-next` daemon crash cause is not yet diagnosed; service is stopped and fully preserved.
- Current production lacks survival, plugin readiness, command/transfer evidence, backup through a shipped operator command, and production-player evidence.
- A disposable PostgreSQL 14 container is available locally and was used for the focused status integration. The full database-backed integration tier has not run and is not a pass.
- Real-player production acceptance requires an authorized online-mode account/client and is not yet attempted.
- Git push/release publication authentication is not yet checked.

## Commands and evidence

All local commands ran from `/home/lkjsxc/workspace/lkjmc`. Full redacted logs are under the three ignored evidence roots named in Repository state.

| Command | Exit | Duration | Classification |
| --- | ---: | ---: | --- |
| `git status --short --branch`; branch/head/log/submodule inventory | 0 | <1s | baseline observed |
| `./scripts/check-lines.py` | 1 | <1s | obsolete 200-line policy rejects replacement `AGENTS.md` |
| `./scripts/check-docs.py` | 1 | <1s | obsolete documentation topology rejects new ledger |
| `cargo fmt --check` | 0 | <1s | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | 8s | pass |
| `cargo test --workspace` | 0 | 16s | pass |
| `python3 tests/lab/test_lab_harness.py` | 0 | <1s | pass |
| `./gradlew --no-daemon --no-build-cache test` | 0 | 5s | pass |
| strict-host-key SSH identity and Incus inventory probes | 0 | 1s | live discovery only |
| Incus snapshots for `lkjmc` and `lkjmc-next` | 0 | 1s | recovery snapshots created |
| first application backup attempt | 1 | 2s | failed safely; no dump created |
| corrected application backup, two DB restores, two file extractions | 0 | 23s | verified preservation |
| restart serving `lkjmc` and correlate three listeners | 0 | 6s | serving historical deployment recovered |
| service-user `lkjmc --json status`, `doctor`, `instance list` | 0 each | 63-74ms | operator observations; not player proof |
| `python3 -m py_compile scripts/check-safe-ops.py scripts/check-fault-harness.py` | 0 | <1s | changed Python valid |
| `python3 scripts/check-safe-ops.py --probe playable-default-secure` | 0 | <1s | retained bind-default property passes |
| `python3 scripts/check-fault-harness.py --probe deterministic-seed-replay` | 0 | 9s | moved executable fixture still owns replay proof |
| `cargo test -p lkjmc-xtask` | 0 | <1s | removed checker commands compile/test |
| first `./scripts/verify-fast.sh` after cleanup | 1 | 10s | old fast gate required a clean worktree inside operations packaging |
| `./scripts/verify-fast.sh` after removing release-only operations packaging from the edit loop | 0 | 3s | deterministic fast tier passes; database/live/Gradle-shadow lanes explicitly skipped |
| independent integrity/truth review | 0 | n/a | found broken obsolete checkers, an unmatched DB-test filter, and fault-fixture gate/status gaps; all accepted findings fixed |
| `cargo test -p lkjmc-store --test safety -- --list` | 0 | 5s | all four exact safety test names used by `check-safe-ops.py` exist; no database behavior ran |
| `python3 scripts/check-fault-harness.py --all` | 0 | 16s | all test-only fault selectors, fixture replay, and release-marker check pass |
| final cleanup review | 0 | n/a | no blocker; found two non-exact download-test filters, then fixed |
| `python3 scripts/check-safe-ops.py --probe atomic-download-faults` | 0 | <1s | exact truncated-download test selected and passed |
| `python3 scripts/check-safe-ops.py --probe partial-final-files-zero` | 0 | <1s | exact concurrent-download test selected and passed |
| final `./scripts/verify-fast.sh` | 0 | 4s | post-review cleanup passes |
| `git diff --check` | 0 | <1s | no whitespace errors |
| focused status Rust tests and target Clippy | 0 | 1-10s | tri-state availability, CLI unknown/truncation, query mapping, and lint pass |
| focused PostgreSQL status test with `--ignored --exact` | 0 | 5-8s | fresh schema, empty view, 33-row deterministic cutoff, diagnostic truncation, and truthful survival row pass |
| disposable PostgreSQL + daemon + Unix-socket CLI JSON/human status | 0 | 8s | four deterministic rows observed: joinable hub, missing-heartbeat backend, proxy, stopped survival; not Minecraft proof |
| `./scripts/check-daemon-cli.sh` | 0 | 1s | no-database Unix-socket status preserves null/unknown and reports no runtime refresh |
| status review and focused follow-up review | 0 | n/a | tri-state, byte bounds, race/deadline, shared-policy, human truncation, explicit DB skip, and future-timestamp findings fixed |
| final status `./scripts/verify-fast.sh` | 0 | 10s | fast tier passes; database/live/Gradle-shadow lanes explicitly skipped |
| `python3 tests/test_release_identity.py` | 0 | 11-12s | five executable regressions cover Cargo warm cache and linked refs, Git/nonce boundaries, modified exports, ambient outputs and output replacement, and compiled JVM mismatch |
| `./scripts/check-operations.py --all --mutations` | 0 | <10s | 121 operation mutations rejected before final provenance marker expansion; final focused artifact-provenance lane rejected 23 mutations |
| `./gradlew --no-daemon --no-build-cache test shadowJar`; `python3 scripts/check-jvm-containment.py --artifacts` | 0 | 2-9s | canonical JVM metadata, generated constants, plugin descriptors, stable artifact names, tests, and shaded closure pass |
| release-identity `./scripts/verify-fast.sh`; `./scripts/check-daemon-cli.sh` | 0 | 9-12s / 1s | Rust workspace and real Unix-socket human/JSON build identity pass |
| independent release-identity review and two follow-ups | 0 | n/a | gitless claims, warm-cache staleness, ambient artifacts, compiled JVM mismatch, human status, fixture escape, output containment, and documentation findings fixed; final review found no release blocker |
| clean `./scripts/build-release.sh <private-external-root>/release` at `34a3338`; independent manifest verify; release secret scan | 0 | 20s | six fresh commit-bound artifacts verified; manifest sidecar SHA-256 `13e8dbe808ad576e65ef3d64abe580d0296749ba67a9c662f7206c07115c1a1c`; disposable bytes removed after evidence copy |
| release CLI, daemon, Discord, and three shaded JVM identity executions | 0 | <1s | all report `0.1.0-alpha.1`, Apache-2.0 where applicable, commit `34a33389adf2f594af5d3a64c02b4ce1d40488f2`, and `dirty=false` |

## Next executable step

Run `rg -n 'denied-unproved|effect_denied' contracts/commands crates/lkjmc-daemon/src crates/lkjmc-cli/src` and delete the first cohesive dormant command domain from contracts, daemon registration, and CLI parsing while retaining status and the mandatory operator/runtime journeys.
