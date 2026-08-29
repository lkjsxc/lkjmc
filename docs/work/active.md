# Active work

## Governing objective

Campaign [`docs/campaigns/202608291859.md`](../campaigns/202608291859.md), **Close Exact Release CI and Disposable Immutable-Update Proof**, governs this checkout. It continues and narrows the predecessor immutable existing-deployment update objective.

Make one exact final commit pass the required clean Compose and remote verification, carry a self-contained importable Git source closure, and produce an independently verified immutable release. Then, only on an authorized isolated unprivileged existing-deployment clone, accept changed update, exact no-op, service and container restart, updater-created PostgreSQL backup restore, and controlled post-fence packaged recovery together for that exact release.

Clean installation, production mutation, public traffic changes, GitHub release publication, and real-player acceptance are outside this campaign.

## Checkout and remote state

- Repository: `lkjsxc/lkjmc` at `/home/coder/workspace/lkjmc`.
- Starting and current branch: `main`; starting and current commit: `09286e5bc57c81afefe51b8b4e9ed1ec849b18ce`.
- After a fresh fetch on 2026-08-29 UTC, `HEAD`, `origin/main`, and the configured upstream are identical (`0` ahead, `0` behind). No newer remote commit or open pull request exists.
- GitHub reports `main` unprotected. Authenticated fetch/workflow observation and eventual direct push are available, but no push is authorized before required local gates pass.
- Pre-task state consists only of the supplied replacement root `AGENTS.md` and supplied untracked campaign archive. There are no staged changes, other tracked/untracked changes, relevant ignored files, submodules, nested repositories, or additional worktrees.
- Installed supplied artifacts: root `AGENTS.md` SHA-256 `a9da2ac00e44c86001fece7e0fb305fe7508f43bbae7546c17b495a7e7c38e5f`; campaign SHA-256 `cdada52f27b4c5f9c92c09b42415ca4c9cf547e7eec320988e01f0cb4dad6f37`. No historical campaign has been edited.
- Rust 1.97.0, OpenJDK 21, PostgreSQL clients 14 and 16, Python 3.12.3, Docker 29.1.3, and Docker Compose 2.40.3 are available. Incus and LXC clients are absent. The local Docker daemon is inside an unprivileged outer system container and OCI container creation is denied at the outer `net.ipv4.ip_unprivileged_port_start` sysctl boundary before a verifier process starts; this is a local environment blocker, not a Compose verification result.

## Current evidence

- `SOURCE INSPECTED`: root policy, supplied campaign, predecessor ledger, README, root workspace metadata, current Verify workflow, and the release/source-closure entrypoints have been reconciled at the starting commit.
- Remote Verify run `31801971914`, rerun attempt 2 job `99090246957`, for exact commit `09286e5bc57c81afefe51b8b4e9ed1ec849b18ce` is `FAILED`. `docs-contracts` passed. The hosted Compose command returned nonzero without the canonical verifier terminal record; because the later release step also failed, no scanned bounded raw verifier log was retained, so the first verifier-owned failure remains unknown. Release construction independently failed while fetching the shallow `HEAD` bundle because parent `7a531a109b9ee0212961493513b3145531e2948e` was absent.
- `INTEGRATION TESTED` at the starting commit outside Docker: the exact clean checkout, a Git-less `git archive` export, PostgreSQL 16, extracted pinned PostgreSQL 14.23, and root execution all reached `ok verify-full`; these observations falsify shallow checkout contents, Git metadata, root identity, and PostgreSQL major version as the hosted verifier failure by themselves. They do not promote the blocked local Compose lane to a pass.
- `IMPLEMENTED`, `UNIT TESTED`: the working tree replaces ambient `HEAD` bundle creation with one clean non-shallow producer, explicit `refs/bundles/lkjmc-source`, and empty-repository import verification. Focused tests accept the complete two-commit closure and reject a shallow producer, an incomplete shallow bundle, an extra advertised ref, and changed exported bytes. `verify-fast` passes on the working tree. Remote and exact committed-source verification remain pending.
- No final release, disposable-host action, protocol-client observation, real-player observation, or production observation has yet been accepted for this campaign.
- Historical ledger claims name `b6d22115f1726aeb570e91900cabcc008ca55689` as a serving baseline and describe prior Incus, PostgreSQL restore, restart, heartbeat, listener, and protocol observations. They are historical inputs only until live revalidated; no host identity, address, snapshot, release, service, database, credential, EULA, route, or traffic fact has been carried forward as current.

## Decisions in force

- Preserve fail-closed exact source, artifact, manifest, ownership, permission, backup, fencing, and recovery checks. A green parser without the verifier owner's canonical success record is not acceptance.
- Export and consume one explicit Git ref with complete object closure; verify it in an empty repository before release construction. Ambient checkout objects, build outputs, ignored files, and caches are not authority.
- Use only the release-packaged update/recovery authority and one global lock. Do not revive the withdrawn checkout installer.
- Do not perform external container, service, database, snapshot, network, or traffic mutation until the authoritative manager, exact disposable target, unprivileged isolation, baseline lineage, capacity, snapshot/backup, rollback, credentials, EULA state, and absence of production traffic are live-discovered and accepted.

## Blockers and untested boundaries

- The first underlying hosted Compose verifier failure before the canonical terminal record remains unresolved. Local Docker cannot execute a container process in this outer unprivileged environment; a new hosted run must retain scanned bounded failure evidence after the source-closure repair.
- Exact source-bundle advertised-ref and complete-object import closure is repaired and focused-tested only in the uncommitted working tree; committed clean-source and remote proof are pending.
- The trusted Ubuntu command-symlink path is deterministic-test evidence only; no current supported-host update boundary has accepted it.
- No authorized disposable host or independently verified healthy retained existing-deployment baseline has yet been discovered from this workspace.
- Changed update, exact no-op, service restart, container restart, updater-created backup restore, controlled interruption, packaged recovery, and disposable network observations are `NOT RUN`.
- Remote green workflow, real-player acceptance, and production observation are `NOT RUN`; the last two are explicitly deferred/forbidden by this campaign.

## Next executable action

Review and commit the source-closure slice, then create and independently import its bundle from that exact clean commit. Push only after that local closure passes; observe the required workflow and download its scanned bounded evidence to identify the first hosted verifier-owned failure if the Compose lane remains nonzero.
