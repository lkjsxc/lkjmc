# Active work

## Governing objective and disposition

Campaign [`docs/campaigns/202608301257.md`](../campaigns/202608301257.md), **Complete the
Disposable Exact-Release Recovery Matrix After the Systemd Fence Repair**, governs this checkout.
It continues, narrows, and supersedes [`docs/campaigns/202608300910.md`](../campaigns/202608300910.md)
for current execution. Campaign `202608300450` remains completed historical context, while
`202608291859` remains completed for release/CI closure and continued only for the unaccepted
disposable operator boundary. No committed predecessor was edited.

The repaired release is independently retrieved, verified, and privately staged. The live matrix is
**BLOCKED before host access or mutation**: this workspace has no noninteractive authenticated
outbound host profile or identity and no Incus/LXD client, remote, or manager socket. The objective is
not accepted; changed update, exact no-op, service restart, container restart, isolated restore,
post-fence restart blocking, and packaged recovery remain unobserved.

## Reconciled checkout and policy

- Repository `lkjsxc/lkjmc` is at `/home/coder/workspace/lkjmc` on branch `main`. The checkout began
  at documentation checkpoint `f2176b284a4b85addbb7de08fe0a3d0fdc680ffc` with a stale local
  `origin/main`; fetching disclosed one fast-forward commit. Before this documentation checkpoint,
  local `HEAD`, fetched `origin/main`, and exact remote `main` agreed at release-under-test commit
  `23ad8d8ef389a453f71ffb3b0a7e333ea1e4a9d4` (`0` ahead, `0` behind). The exact final
  campaign/evidence checkpoint is the documentation-only successor containing this ledger and is read
  from repository `HEAD`; it is not the installed or running release identity.
- Initial state contained no staged or unstaged tracked change and exactly one supplied untracked
  campaign. Relevant ignored state was bounded Python `__pycache__` output under `scripts/` and
  `tests/`. There was one worktree, no submodule, and no nested repository. The temporary detached
  verifier worktree was removed after use.
- Supplied campaign `/home/coder/workspace/lkjmc/docs/campaigns/202608301257.md` is installed
  unchanged, SHA-256 `14d2242c9ba654ca69428ff0efb30042b310c4cd61caf290f3e14cbf1fc99a31`.
  Root `/home/coder/workspace/lkjmc/AGENTS.md` is byte-identical to tracked and supplied durable
  policy, SHA-256 `38bfe676b1f6b964f06854a85e021634f1c7d24168b09b21ada97e51fafdc193`;
  no durable policy changed.
- GitHub authentication and workflow read access are current. `main` has no branch protection or
  ruleset, and the public repository has no open issue or pull request. Git 2.43.0, Rust 1.97.0,
  Cargo 1.97.0, Python 3.12.3, Java 21.0.12, PostgreSQL client tools 16.15, OpenSSH 9.6p1, and local
  noninteractive sudo are available.
- The repaired effective-systemd-command regression passed locally (`1` focused test). The exact
  remote required workflow is green; no preexisting objective-relevant deterministic failure was
  found. Source was not changed in this campaign.

## Exact repaired artifact evidence

- Required `Verify` run `33288687707`, attempt `1`, event `push`, ref `refs/heads/main`, exact head
  `23ad8d8ef389a453f71ffb3b0a7e333ea1e4a9d4`, and jobs `docs-contracts`, `verify-compose`, and
  `verify-release-artifact` are currently `success`.
- Release artifact ID `9725523129` is named
  `lkjmc-release-23ad8d8ef389a453f71ffb3b0a7e333ea1e4a9d4-run-33288687707-attempt-1`, is
  unexpired through `2026-09-29T03:08:05Z`, and is `23,539,404` bytes. Fresh exact-ID API retrieval
  recomputed raw outer SHA-256
  `eeed00ebd5d7dbf3263ff2afaf7b9f12b45ba7632320773bc936a18c4da5a70a`, equal to the artifact
  service digest.
- The raw ZIP has exactly three canonical regular members: the USTAR, its checksum sidecar, and
  `release-handoff.json`. The canonical owner verifies archive
  `lkjmc-0.1.0-alpha.1-23ad8d8ef389a453f71ffb3b0a7e333ea1e4a9d4.tar`, size `23,537,664`,
  SHA-256 `7a5c98b0fc066e7f9930562e4f8d5ce71443691e097c32d9d331a5ddcf3e7df8`, one top-level
  directory, eighteen explicit members, normalized metadata, and no link, special file, traversal,
  duplicate, extra, or mode difference.
- Safe extraction has fourteen declared installed artifacts and sixteen regular files at exact
  `0600`/`0700` modes. Release-manifest SHA-256 is
  `ec91332d49f5ba61f991b4bd4767007e3b53e89f33f898a6aaaf1ec71c48af8c`; manifest-sidecar-file
  SHA-256 is `5a8b6a7d173aea6790b5520e4c3c006c17205077bc99050e91de61c45e3fb175`.
  Independent Rust/JVM identity is version `0.1.0-alpha.1`, commit
  `23ad8d8ef389a453f71ffb3b0a7e333ea1e4a9d4`, clean.
- Canonical `verify`, `consume`, `extract`, independent manifest verification, independent built
  identity, and a six-root nested-archive secret scan passed without Cargo/Gradle execution or a
  release rebuild. Consumer-receipt artifact ID `9725525617`, raw digest
  `c3aca0181ef77a6219067543157b413fd61ca66a1dbd4de5501d0c0097540a7a`, size `582`, and expiry
  `2026-09-29T03:08:18Z` independently download to the byte-identical canonical receipt generated
  locally.
- Private operator state is retained at `/tmp/lkjmc-202608301257-operator.BbtjLv` (`0700`, regular
  files `0600`). It contains the exact raw artifacts, API metadata, bounded verification output, and
  extracted release root, but no host credential or deployment state. The current operator owns it;
  remove it after live transfer and matrix acceptance or when this campaign is explicitly abandoned.
  The predecessor workspace `/tmp/lkjmc-202608300910-operator.OY8ekk` remains separately retained by
  its existing owner and removal condition; its pre-repair artifact is historical evidence and is not
  eligible for live reuse.

## External boundary and untested work

- This workspace is itself an unprivileged LXC container with shifted IDs, not a manager. Incus,
  LXD, and `lxc` clients and local manager sockets are absent. The user account has no SSH config,
  private key, or agent; its four known-host entries are hashed and do not provide an authenticated
  target. Repository Actions has no configured secret and no self-hosted runner that supplies a host
  path.
- No host, manager, source deployment, disposable clone, snapshot, database, credential copy,
  listener, route, service, public traffic, player, or production state was accessed or mutated. No
  disposable cleanup target exists.
- **SOURCE INSPECTED**, **UNIT TESTED** (focused parser regression), **GENERATED ARTIFACT VERIFIED**,
  and **RELEASE ARTIFACT VERIFIED** are current. Artifact retrieval is **OPERATOR OBSERVED**.
  **FRESH SUPPORTED-HOST INSTALLED**, **DISPOSABLE NETWORK OBSERVED**, changed update, exact no-op,
  both restart rows, PostgreSQL restore, interruption, fencing, packaged recovery, and
  **PROTOCOL-CLIENT OBSERVED** are **BLOCKED** behind host access. **REAL-PLAYER OBSERVED** and
  **PRODUCTION OBSERVED** are **NOT RUN** and remain deferred or forbidden.

## Next executable action

Activate a noninteractive authenticated connection profile for the already authorized home-server
manager. Then live-discover whether Incus or LXD is authoritative and identify the exact healthy
source deployment, capacity, isolation, consent, and rollback prerequisites read-only before any
clone, transfer, or mutation.
