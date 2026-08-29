# Active work

## Governing objective

Campaign [`docs/campaigns/202608291859.md`](../campaigns/202608291859.md), **Close Exact Release CI and Disposable Immutable-Update Proof**, governs this checkout. It continues and narrows the predecessor immutable existing-deployment update objective.

Make one exact final commit pass the required clean Compose and remote verification, carry a self-contained importable Git source closure, and produce an independently verified immutable release. Then, only on an authorized isolated unprivileged existing-deployment clone, accept changed update, exact no-op, service and container restart, updater-created PostgreSQL backup restore, and controlled post-fence packaged recovery together for that exact release.

Clean installation, production mutation, public traffic changes, GitHub release publication, and real-player acceptance are outside this campaign.

## Checkout and remote state

- Repository: `lkjsxc/lkjmc` at `/home/coder/workspace/lkjmc`.
- Starting and current branch: `main`; starting commit: `09286e5bc57c81afefe51b8b4e9ed1ec849b18ce`; latest exact remotely observed predecessor: `ff442cb5874a48a1bfc319be602000a8d704c2a7`. Current source is the commit containing this ledger.
- On 2026-08-29 UTC, `HEAD`, `origin/main`, and the configured upstream are identical (`0` ahead, `0` behind). No open pull request exists.
- GitHub reports `main` unprotected. Authenticated direct pushes and exact workflow observation are available and have been used only after local focused gates.
- Pre-task state consists only of the supplied replacement root `AGENTS.md` and supplied untracked campaign archive. There are no staged changes, other tracked/untracked changes, relevant ignored files, submodules, nested repositories, or additional worktrees.
- Installed supplied artifacts: root `AGENTS.md` SHA-256 `a9da2ac00e44c86001fece7e0fb305fe7508f43bbae7546c17b495a7e7c38e5f`; campaign SHA-256 `cdada52f27b4c5f9c92c09b42415ca4c9cf547e7eec320988e01f0cb4dad6f37`. No historical campaign has been edited.
- Rust 1.97.0, OpenJDK 21, PostgreSQL clients 14 and 16, Python 3.12.3, Docker 29.1.3, and Docker Compose 2.40.3 are available. Incus and LXC clients are absent. The local Docker daemon is inside an unprivileged outer system container and OCI container creation is denied at the outer `net.ipv4.ip_unprivileged_port_start` sysctl boundary before a verifier process starts; this is a local environment blocker, not a Compose verification result.

## Current evidence

- `SOURCE INSPECTED`: root policy, supplied campaign, predecessor ledger, README, root workspace metadata, current Verify workflow, and the release/source-closure entrypoints have been reconciled at the starting commit.
- The starting remote run failed both before the canonical Compose terminal record and while importing a shallow `HEAD` bundle. Commits through `a648928a0e105457097b92e695fa7227c5b31829` replace that bundle with complete explicit `refs/bundles/lkjmc-source` closure, preserve exported source modes, compact the verifier below the unchanged 2 GiB archive bound, and validate Docker's `scratch`, internal stages, optional metadata, and platform-pruned OCI indexes. Hosted source export, strict bundle attachment, fresh release construction, embedded identity, and the exact 14-artifact manifest now pass together.
- Remote Verify run `33253846816`, Compose job `99104056058`, for exact commit `a648928a0e105457097b92e695fa7227c5b31829` is `FAILED`; `docs-contracts`, source export/import/attachment, release build, embedded identity, manifest closure, cleanup, and secret scanning passed. Its retained redacted log identifies the first verifier failure: the PostgreSQL-backed slow support-bundle test returned after its one-second effect cap but exceeded an additional 250 ms wall-clock assertion under four-thread hosted contention. The source containing this ledger retains the one-second cap, gives host scheduling one explicit second of margin that remains below the injected five-second stall, reports elapsed time on failure, passes five focused PostgreSQL repetitions, and passes the full daemon binary (`204` passed, `2` intentionally ignored) with four test threads.
- The same exact run's independent image audit rejected `blobs/sha256/295a38ad280f4ced87613fd5b0db025b016a80d669f43a753d2b51148f4df061` as unreferenced. Current Moby 28 classic-store source shows that `docker save` emits content-addressed synthetic V1 configs for every layer prefix in addition to the Docker/OCI closure; Docker 29's containerd store does not emit them. The source containing this ledger accepts only a unique recomputed V1 `id`/`parent` chain matching each declared diff-ID sequence and terminal image config. A two-layer/shared-layer fixture passes; missing, detached, altered, ambiguous, type-invalid, schema-expanded, config-mismatched, and digest-mismatched chains fail, and unrelated extras remain fatal.
- Remote Verify run `33256284139`, Compose job `99110479008`, for exact commit `ff442cb5874a48a1bfc319be602000a8d704c2a7` is `FAILED`; `docs-contracts`, source import/attachment, release construction, embedded identity, the exact 14-artifact manifest, cleanup, and the full secret scan passed. The support-bundle timing repair passed. The next verifier failure was a process test that signalled a group with `SIGKILL` and immediately asserted the asynchronous post-kill observation; the source containing this ledger waits up to two seconds for the real absent observation and otherwise reports the last state. Ten focused PostgreSQL-backed repetitions and the full four-thread daemon binary pass (`204` passed, `2` intentionally ignored).
- The same run's release audit recomputed no matching four-layer legacy chain. An isolated no-network Docker 29 classic-store daemon proved the chain IDs correct and exposed the exact mismatch: Moby expands sparse OCI `config` into typed V1 config with explicit zero fields. The source containing this ledger requires every source field to match, permits only zero-valued typed additions, and rejects nonzero additions. Real one-layer and 11-layer classic-store archives pass with exact `1` and `11` legacy-config closures; the multi-image shared-layer fixture and all negative mutations pass.
- `FORMATTED`, `UNIT TESTED`, `INTEGRATION TESTED`, `POSTGRESQL TESTED`: three official Go `encoding/json` identity vectors, the semantic saved-image suite, all eight operations probes, all `160` operations mutations, ten focused adoption repetitions, the full daemon binary, and the final 11-layer classic-store archive pass. Fresh `verify-fast` also passes with only its declared database-backed, live-smoke, and Gradle `shadowJar` skips.
- `INTEGRATION TESTED` outside Docker: exact clean/Git-less source verification, real PostgreSQL, complete explicit source-bundle import, a real digest-pinned PostgreSQL saved-image export, and Docker 29 containerd saved-image metadata have passed their affected local boundaries. Local ordinary container creation remains blocked by the outer unprivileged container's read-only `net.ipv4.ip_unprivileged_port_start`; it is not promoted to Compose evidence.
- No final release, disposable-host action, protocol-client observation, real-player observation, or production observation has yet been accepted for this campaign.
- Historical ledger claims name `b6d22115f1726aeb570e91900cabcc008ca55689` as a serving baseline and describe prior Incus, PostgreSQL restore, restart, heartbeat, listener, and protocol observations. They are historical inputs only until live revalidated; no host identity, address, snapshot, release, service, database, credential, EULA, route, or traffic fact has been carried forward as current.

## Decisions in force

- Preserve fail-closed exact source, artifact, manifest, ownership, permission, backup, fencing, and recovery checks. A green parser without the verifier owner's canonical success record is not acceptance.
- Export and consume one explicit Git ref with complete object closure; verify it in an empty repository before release construction. Ambient checkout objects, build outputs, ignored files, and caches are not authority.
- Use only the release-packaged update/recovery authority and one global lock. Do not revive the withdrawn checkout installer.
- Do not perform external container, service, database, snapshot, network, or traffic mutation until the authoritative manager, exact disposable target, unprivileged isolation, baseline lineage, capacity, snapshot/backup, rollback, credentials, EULA state, and absence of production traffic are live-discovered and accepted.

## Blockers and untested boundaries

- The two exact `ff442cb5` hosted failures are resolved locally but not yet remotely observed. Local Docker still cannot execute a container process in this outer unprivileged environment, so the exact Compose terminal record remains remote-only evidence.
- Exact source-bundle advertised-ref and complete-object import closure is committed, pushed, independently imported, and accepted by hosted bundle attachment and release construction. The exact 14-artifact release manifest built remotely at `ff442cb5`, but the normalized classic-store image audit and full verifier have not yet passed together.
- The trusted Ubuntu command-symlink path is deterministic-test evidence only; no current supported-host update boundary has accepted it.
- No authorized disposable host or independently verified healthy retained existing-deployment baseline has yet been discovered from this workspace.
- Changed update, exact no-op, service restart, container restart, updater-created backup restore, controlled interruption, packaged recovery, and disposable network observations are `NOT RUN`.
- Remote green workflow, real-player acceptance, and production observation are `NOT RUN`; the last two are explicitly deferred/forbidden by this campaign.

## Next executable action

Commit the bounded adoption observation and strict typed-V1 normalization after fresh local gates, push them, and observe the exact workflow. Require the Compose verifier's canonical terminal record, exact release closure, image/evidence scans, and cleanup to pass together before beginning final release or disposable-host work.
