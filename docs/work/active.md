# Active work

## Governing objective

Campaign [`docs/campaigns/202608312120.md`](../campaigns/202608312120.md), **Rust-Owned Fresh
Supported-Container Installation and Operator Acceptance**, governs this checkout. Its immediate
predecessor, [`docs/campaigns/202608311119.md`](../campaigns/202608311119.md), is complete only at
the source, deterministic, PostgreSQL/process, retained-release, and independent-retrieval tiers.
The earlier fixed-topology campaign is superseded for implementation and retained only for recovery
scenario semantics.

The active objective is one packaged Rust `lkjmc-ops` first-install authority for a prepared,
unprivileged systemd container. It must anchor an exact release and immutable asset closure, create
only exact lkjmc-owned state, initialize PostgreSQL and the typed fleet, activate the service, and
truthfully classify exact replay, resume, conflict, and acceptance. The release must first stop
shipping the withdrawn `lkjmc-discord` executable and its direct current consumers.

## Reconciled checkout

- Repository: `lkjsxc/lkjmc`, `/home/coder/workspace/lkjmc`; branch `main` at
  `13602f277bcc109d891bc3726976a0323936e8b3`. `origin/main` resolved to the same commit on
  2026-08-31; local ahead/behind count was `0/0`, and an authenticated fast-forward push dry run
  reported no change. `main` is currently reported unprotected by GitHub.
- One worktree exists; there are no submodules, nested repositories, or subtree `AGENTS.md` files.
  Before campaign edits, the worktree contained the user-supplied modified root `AGENTS.md` and the
  user-supplied untracked governing campaign; no tracked source changes were present. Relevant
  ignored paths are build and bounded test-cache output only. `git diff --check`, staged diff check,
  and `git fsck --no-dangling` passed before edits.
- Current supplied policy SHA-256:
  `96a3188351b086769f533d8de4d75bfcd596401a3fff98c4f627e0f625382299` (`AGENTS.md`). Current
  campaign SHA-256:
  `c61f63db77c3f6817f63fa3cf94c18785db29db9e9b373715371618989d6fa2e`
  (`docs/campaigns/202608312120.md`). The pre-edit ledger SHA-256 was
  `81b61c6b4cc32bea0ed10dfe0febca1a09ed056354c0309fbb6dedf0be98f792`.
- Native Rust, Java, Gradle, PostgreSQL client, Incus, and LXD executables are absent from this
  checkout host. Docker, systemd utilities, fixed identity utilities, GitHub CLI, and an authenticated
  GitHub account are available. Docker is not supported-host evidence and will not be substituted for
  an Incus/LXD container.

## Revalidated predecessor evidence

- GitHub `Verify` run `33373201044` for `13602f277bcc109d891bc3726976a0323936e8b3` completed
  successfully on 2026-08-31. Its release artifact `9751489948` remains unexpired through
  2026-09-30T08:52:34Z; the consumer receipt artifact remains unexpired through
  2026-09-30T08:52:54Z.
- The release and its separate consumer receipt were independently downloaded locally. The inner
  archive SHA-256 is
  `f8331ffa07ee2f94f0cafe462b1094bc7fdd158dafd3e5a805863f7f91b1f700`, matching its sidecar and
  receipt. The receipt records manifest SHA-256
  `c332af627a4cb2fc60856cb3be981f78821dc75c3f9044303c1539e523c590cd`, artifact-service digest
  `sha256:e1891b0f65e3586e86d1d272f510fb8461d062cdc10deac46da00b3523130717`, and successful
  nine-artifact/112-contract verification.
- This is **RELEASE ARTIFACT VERIFIED** and independent retrieval evidence for the predecessor. It
  is not installation, systemd, supported-host, operator, protocol-client, real-player, or production
  evidence. Those boundaries remain **NOT RUN** for that release.

## Current decisions and evidence limits

- The final release must contain only the credible core: current baseline inventory inspection found
  four native binaries, including withdrawn `lkjmc-discord`, three Java jars, and two declarative
  systemd files. `lkjmc-discord` is a strict first cutover target; historical migrations and unrelated
  Discord data are deferred unless a current owner proves them necessary for fresh installation.
- First install will remain one direct typed `lkjmc-ops` command, inside a prepared supported
  substrate. It will reuse the existing anchored manifest, atomic artifact publication, durable
  journal/fence, EULA, fleet, database, and post-start owners rather than creating another installer
  or fleet model.
- No Incus/LXD manager or authorized supported-container boundary has been discovered. Supported-host,
  operator, protocol-client, service/container restart, and isolated-restore observations are
  **NOT RUN** at this point; they may become **BLOCKED** only after the independent implementation and
  release work is complete if no authorized manager is available.

## Next executable action

Complete the focused `lkjmc-discord` consumer inventory, then delete the withdrawn executable and
every direct release, build, verification, configuration, test, and current-document consumer before
defining the first-install input contract.
