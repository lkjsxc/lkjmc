# Active work

## Governing objective and disposition

Campaign [`docs/campaigns/202608300910.md`](../campaigns/202608300910.md), **Prove
Disposable Exact-Release Update, Restart, Restore, and Recovery**, governs this checkout. It accepts
and supersedes [`docs/campaigns/202608300450.md`](../campaigns/202608300450.md) for current execution
and continues only the disposable live boundary of
[`docs/campaigns/202608291859.md`](../campaigns/202608291859.md). Neither predecessor was edited.

The exact release artifact is independently verified and retained locally. The supported-host live
matrix is **BLOCKED before host access or mutation**: this workspace has no authenticated outbound
SSH identity or configured host profile and no Incus/LXD client, remote, or manager socket. A usable
authenticated path to the already authorized home-server manager is required; no hostname or target
will be guessed from historical evidence.

## Reconciled checkout and policy

- Repository `lkjsxc/lkjmc` is at `/home/coder/workspace/lkjmc`, branch `main`, starting `HEAD`
  `97084332d3d00b9c44a11c7dfc4bbd4ba26226f6`. Fetched `origin/main`, `ls-remote`, and local `HEAD`
  agree (`0` ahead, `0` behind). The public repository reports unprotected `main`, no ruleset, and
  no open issue or pull request; authenticated direct push and workflow API access are available.
- Initial state was exactly one supplied untracked campaign and no staged or unstaged tracked change,
  ignored file, submodule, nested repository, or additional worktree. The temporary clean verifier
  worktree was removed after use. The campaign-containing evidence checkpoint is a documentation-only
  successor; its exact ID is read from repository `HEAD` after integration.
- Supplied campaign `/home/coder/workspace/lkjmc/docs/campaigns/202608300910.md` is installed unchanged,
  SHA-256 `5a01fbb9dfc33880b915747c858947a5a5df1cd143a18e5cad78efa11144a528`.
  Root `/home/coder/workspace/lkjmc/AGENTS.md` is byte-identical to tracked source, SHA-256
  `38bfe676b1f6b964f06854a85e021634f1c7d24168b09b21ada97e51fafdc193`; no policy replacement was
  needed and no durable policy changed.
- Git 2.43.0, Python 3.12.3, GitHub CLI 2.45.0, OpenSSH 9.6p1, Java 21.0.12, PostgreSQL client tools
  16.15, and noninteractive local sudo are available. Incus and LXD clients are absent locally.
- No preexisting relevant deterministic failure was found: the focused archive, identity, and
  deployer suites pass all 35 tests. One expected clean-source guard rejected verification from the
  active dirty checkout; the same canonical command passed from a clean detached worktree at the
  artifact commit without changing source.

## Exact artifact evidence

- Required `Verify` run `33278913861`, attempt `2`, event `push`, ref `refs/heads/main`, exact head
  `97084332d3d00b9c44a11c7dfc4bbd4ba26226f6`, and jobs `docs-contracts`, `verify-compose`, and
  `verify-release-artifact` are currently `success`.
- Release artifact ID `9722831162` is named
  `lkjmc-release-97084332d3d00b9c44a11c7dfc4bbd4ba26226f6-run-33278913861-attempt-2`, is unexpired
  through `2026-09-28T23:14:07Z`, and is `23,538,892` bytes. Exact-ID API retrieval recomputed raw
  outer SHA-256 `fb5c51a5b8d971741c2942f9642b0a1ae4179ada0049a808c4f28c94f29d192d`, equal to the artifact
  service digest.
- The raw ZIP has exactly three canonical regular members: the USTAR, its checksum sidecar, and
  `release-handoff.json`. The canonical owner verifies archive
  `lkjmc-0.1.0-alpha.1-97084332d3d00b9c44a11c7dfc4bbd4ba26226f6.tar`, size `23,537,152`, SHA-256
  `55d9fe64a319b67c7aa02e5391ed15c8a7ebb0b3cdb8ef98cb5334ebf059de71`, one top-level directory,
  eighteen explicit members, normalized metadata, and no link, special file, traversal, duplicate,
  extra, or mode difference.
- Safe extraction has fourteen declared installed artifacts and sixteen regular files at exact
  `0600`/`0700` modes. Release-manifest SHA-256 is
  `7c2526237cf9c76e7be5610136391e68691602329ee7c98dc61a5ccd49cbfdb4`; manifest-sidecar-file
  SHA-256 is `8e5ea285808896cca6a023fe589dfff9e0f0dedf9e9be10dbdd4c9ecebad03fa`. Independent Rust/JVM
  identity is version `0.1.0-alpha.1`, commit
  `97084332d3d00b9c44a11c7dfc4bbd4ba26226f6`, clean.
- Canonical `verify`, `consume`, `extract`, independent manifest verification, independent built
  identity, and a six-root secret scan all passed without Cargo/Gradle execution or release rebuild.
  Retained consumer-receipt artifact ID `9722834486`, raw digest
  `9a6b59a572635179a00238209cb78228c6b9ce0000c9bb0dadd7467dd904dce2`, independently downloads
  to the exact same canonical receipt generated locally.
- Private operator state is retained at `/tmp/lkjmc-202608300910-operator.OY8ekk` (`0700`, 38 MiB;
  regular evidence and artifact files `0600`). The current operator owns it; remove it after live
  transfer/reverification or when the campaign is explicitly abandoned. It contains no copied host
  credential or deployment state.

## External boundary and untested work

- Read-only local discovery identifies this workspace itself as an unprivileged LXC container
  (shifted UID map), not an Incus/LXD manager. It has no manager binary, package, service, state path,
  client configuration, or socket; it also has no lkjmc release root, service, configuration, data,
  deployment journal, fence, permit, or backup.
- The user account has no SSH config, key, or agent. Root has only an inbound `authorized_keys` file;
  no outbound SSH or container-manager client configuration was found. Historical hostnames and
  deployment identifiers were not used as connection targets.
- No source deployment, disposable clone, snapshot, database, credential copy, listener, route,
  service, public traffic, player, or production state was accessed or mutated. No temporary restore
  database or external cleanup target exists.
- `SOURCE INSPECTED`, `UNIT TESTED` (35 focused tests), and `RELEASE ARTIFACT VERIFIED` are current.
  Artifact-retrieval `OPERATOR OBSERVED` is current. `FRESH SUPPORTED-HOST INSTALLED`, disposable
  network, changed update, exact no-op, service restart, container restart, updater-backup restore,
  post-fence interruption, fenced restart, packaged recovery, and protocol-client observation are
  `BLOCKED` behind host access. Real-player and production observation are `NOT RUN` and remain
  deferred or forbidden.

## Next executable action

Provide or activate a noninteractive authenticated connection profile for the authorized home server.
Then live-discover whether Incus or LXD is authoritative, identify the exact healthy deployment and
capacity read-only, and prove clone isolation and rollback prerequisites before any mutation. Do not
transfer or update until those facts pass.
