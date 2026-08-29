# Active work

## Governing objective

Campaign [`docs/campaigns/202608291859.md`](../campaigns/202608291859.md), **Close Exact Release CI and Disposable Immutable-Update Proof**, governs this checkout. It continues and narrows the predecessor immutable existing-deployment update objective.

Make one exact final commit pass the required clean Compose and remote verification, carry a self-contained importable Git source closure, and produce an independently verified immutable release. Then, only on an authorized isolated unprivileged existing-deployment clone, accept changed update, exact no-op, service and container restart, updater-created PostgreSQL backup restore, and controlled post-fence packaged recovery together for that exact release.

Clean installation, production mutation, public traffic changes, GitHub release publication, and real-player acceptance are outside this campaign.

## Checkout and remote state

- Repository: `lkjsxc/lkjmc` at `/home/coder/workspace/lkjmc`.
- Starting and current branch: `main`; starting commit: `09286e5bc57c81afefe51b8b4e9ed1ec849b18ce`; latest exact remotely verified commit: `5e5ece101ec1c3e45713a09f0a644095052a749d`. The active reproducibility slice is the source containing this ledger.
- On 2026-08-29 UTC, `HEAD`, `origin/main`, and the configured upstream are identical (`0` ahead, `0` behind). No open pull request exists.
- GitHub reports `main` unprotected. Authenticated direct pushes and exact workflow observation are available and have been used only after local focused gates.
- Pre-task state consists only of the supplied replacement root `AGENTS.md` and supplied untracked campaign archive. There are no staged changes, other tracked/untracked changes, relevant ignored files, submodules, nested repositories, or additional worktrees.
- Installed supplied artifacts: root `AGENTS.md` SHA-256 `a9da2ac00e44c86001fece7e0fb305fe7508f43bbae7546c17b495a7e7c38e5f`; campaign SHA-256 `cdada52f27b4c5f9c92c09b42415ca4c9cf547e7eec320988e01f0cb4dad6f37`. No historical campaign has been edited.
- Rust 1.97.0, OpenJDK 21, PostgreSQL clients 14 and 16, Python 3.12.3, Docker 29.1.3, and Docker Compose 2.40.3 are available. Incus and LXC clients are absent. The local Docker daemon is inside an unprivileged outer system container and OCI container creation is denied at the outer `net.ipv4.ip_unprivileged_port_start` sysctl boundary before a verifier process starts; this is a local environment blocker, not a Compose verification result.

## Current evidence

- `SOURCE INSPECTED`: root policy, supplied campaign, predecessor ledger, README, root workspace metadata, current Verify workflow, and the release/source-closure entrypoints have been reconciled at the starting commit.
- The starting failure was reproduced and closed without weakening evidence: the workflow now exports complete history under the sole explicit `refs/bundles/lkjmc-source` ref, imports it into an empty repository, and verifies exact source/export equality before release construction. The exact `5e5ece10` bundle SHA-256 is `a6508f27d27f233df600d40fd1c5026fa13577fba61922bada70d17edfeacd86`.
- Subsequent hosted failures exposed and led to bounded repairs for verifier archive size, a PostgreSQL test's scheduling allowance, strict Moby classic-store typed V1 config normalization, and an asynchronous post-`SIGKILL` observation race. Negative archive and operation mutations remain fail-closed.
- `INTEGRATION TESTED`, `POSTGRESQL TESTED`, `GENERATED ARTIFACT VERIFIED`, `RELEASE ARTIFACT VERIFIED`: GitHub Verify run [`33257611660`](https://github.com/lkjsxc/lkjmc/actions/runs/33257611660), Compose job `99113956153`, is successful for exact commit `5e5ece101ec1c3e45713a09f0a644095052a749d`. The clean Compose verifier reached its canonical terminal record with `50` completed checks and `3` explicit live-only skips; source import/attachment, release construction, exact `14`-artifact manifest, `112` contracts, `215` components, `3` images, retained-evidence closure, full secret scan, and cleanup all passed. Cleanup independently reported zero remaining project containers, networks, and volumes.
- `FORMATTED`, `UNIT TESTED`, `INTEGRATION TESTED`, `POSTGRESQL TESTED`: at `5e5ece10`, fresh `verify-fast`, all eight operations probes and `160` mutations, the semantic saved-image suite, official Go JSON identity vectors, ten focused adoption repetitions, the full four-thread daemon binary (`204` passed, `2` intentionally ignored), and a real isolated Docker 29 classic-store 11-layer archive passed. Local ordinary container creation remains blocked by the outer unprivileged container's read-only `net.ipv4.ip_unprivileged_port_start`; hosted Compose evidence is not attributed to the local environment.
- Two independent exact `5e5ece10` release builds each passed embedded identity and manifest verification, but byte comparison found the three shaded JVM JARs unequal. Every unpacked entry was identical; ZIP timestamps and traversal order differed. The active source disables timestamp preservation and enables reproducible file order on every Gradle `Jar` task, makes those settings part of the artifact-provenance operations contract, and documents that unpacked equality is insufficient. Two clean no-cache `shadowJar` builds now produce identical hashes for all three JARs, with every entry timestamp normalized to `1980-02-01 00:00:00`; the focused provenance probe rejects `42` mutations.
- No final release, disposable-host action, protocol-client observation, real-player observation, or production observation has yet been accepted for this campaign.
- Historical ledger claims name `b6d22115f1726aeb570e91900cabcc008ca55689` as a serving baseline and describe prior Incus, PostgreSQL restore, restart, heartbeat, listener, and protocol observations. They are historical inputs only until live revalidated; no host identity, address, snapshot, release, service, database, credential, EULA, route, or traffic fact has been carried forward as current.

## Decisions in force

- Preserve fail-closed exact source, artifact, manifest, ownership, permission, backup, fencing, and recovery checks. A green parser without the verifier owner's canonical success record is not acceptance.
- Export and consume one explicit Git ref with complete object closure; verify it in an empty repository before release construction. Ambient checkout objects, build outputs, ignored files, and caches are not authority.
- Use only the release-packaged update/recovery authority and one global lock. Do not revive the withdrawn checkout installer.
- Do not perform external container, service, database, snapshot, network, or traffic mutation until the authoritative manager, exact disposable target, unprivileged isolation, baseline lineage, capacity, snapshot/backup, rollback, credentials, EULA state, and absence of production traffic are live-discovered and accepted.

## Blockers and untested boundaries

- Exact source/Compose/release CI is green at `5e5ece10`, but the active Gradle reproducibility repair changes release inputs. That evidence is historical for the predecessor until fresh local release comparison and the required workflow pass for the final commit.
- The trusted Ubuntu command-symlink path is deterministic-test evidence only; no current supported-host update boundary has accepted it.
- No authorized disposable host or independently verified healthy retained existing-deployment baseline has yet been discovered from this workspace.
- Changed update, exact no-op, service restart, container restart, updater-created backup restore, controlled interruption, packaged recovery, and disposable network observations are `NOT RUN`.
- Real-player acceptance and production observation are `NOT RUN` and explicitly deferred/forbidden by this campaign.

## Next executable action

Run fresh local gates, commit the JVM archive reproducibility repair, construct and compare two complete releases from that exact clean commit, then push and require the exact remote workflow to pass. In parallel only through read-only discovery, determine whether an authorized isolated existing-deployment baseline is available; if not, classify the disposable tier `BLOCKED` without creating an unsupported installation or touching production.
