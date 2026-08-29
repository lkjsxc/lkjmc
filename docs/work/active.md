# Active work

## Governing objective

Campaign [`docs/campaigns/202608300450.md`](../campaigns/202608300450.md), **Promote and Reconsume the Exact Verified Release Artifact**, governs this checkout. It continues and narrows [`docs/campaigns/202608291859.md`](../campaigns/202608291859.md): the predecessor's exact-source, pinned-build, two-root reproducibility, manifest, secret-scan, cleanup, and required remote-CI slice is accepted at the inspected revision, while its disposable update/no-op/restart/restore/recovery slice remains unobserved and is deferred.

For one exact final `main` commit, package one already compared release root as a deterministic permission-preserving POSIX `ustar`, retain its exact three-file handoff closure through GitHub Actions, consume it in a separate required same-run job without rebuilding, and independently download and reverify it after the run. This campaign is release handoff only; it does not install or deploy the bytes.

## Reconciled checkout and external boundary

- Repository: `lkjsxc/lkjmc` at `/home/coder/workspace/lkjmc`; branch `main`; starting and current `HEAD` `2d14dfa78ec0ab6c270c9e652c98517bca034c8f`.
- `HEAD`, configured upstream `origin/main`, fetched remote `main`, and `ls-remote` agree (`0` ahead, `0` behind). The remote has not moved since campaign inspection. The branch is unprotected, no repository rulesets apply, no pull request or issue is open, and authenticated direct push plus workflow/artifact API access are available.
- Pre-task state is exactly the supplied modified root `AGENTS.md` and supplied untracked campaign. Nothing is staged; there are no other tracked/untracked changes, relevant ignored files, submodules, nested repositories, or additional worktrees.
- Installed supplied inputs: `/home/coder/workspace/lkjmc/AGENTS.md` SHA-256 `38bfe676b1f6b964f06854a85e021634f1c7d24168b09b21ada97e51fafdc193`; `/home/coder/workspace/lkjmc/docs/campaigns/202608300450.md` SHA-256 `0aa81a4e53e6b1a0bc978ef69c515f718fe48238383ed31a7af656f6fa532688`. No committed historical campaign was edited.
- Python 3.12.3, Git 2.43.0, Docker client/server 29.1.3, Docker Compose 2.40.3, GitHub CLI 2.45.0, and authenticated `repo`/`workflow` scopes are available. Local nested-container usability has not yet been re-observed for this campaign.
- The exact inspected push run `33263756981`, attempt `1`, is successful for the starting commit. Its only retained artifact is `operations-evidence` ID `9718299854`, service digest `sha256:8d51ad5b9eca87977f95d1dca96c036a66b115fc9409249634c2e2fe7dccc955`, expiring `2026-11-27T16:43:53Z`; independent redownload confirms ten outer files but no release root or transport archive.
- That retained manifest binds the starting commit, declares fourteen installed artifacts totaling `23,453,728` bytes, and has SHA-256 `9c08ea6ab90f08c8a2db2b3268badd31c84acd9fe411e9273761af126ba79e53`. The sidecar file SHA-256 is `343f09d940cb840e4877e251bdcf87ca2223e490481fe5b042be753523e96e36`. These are historical starting-run identities, not the future final archive.

## Current owner and action findings

- `config/release-artifacts.json` owns the fourteen installed artifacts; `scripts/build-release.sh` owns the release root; `scripts/artifact-manifest.py` and `scripts/verify-artifact-manifest.py` own manifest generation and independent verification; `scripts/compare-release-roots.py` owns two-root byte/mode equality; `scripts/fd_tree.py` is the existing descriptor-relative no-follow walker.
- No canonical release transport packer, retained release-byte artifact, `download-artifact` use, separate artifact-consumer job, or competing handoff prototype exists in the active checkout.
- The pinned upload action `actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02` exposes `artifact-id`, `artifact-url`, and `artifact-digest`, defaults to collision refusal, and supports explicit retention. Its normal outer ZIP does not preserve executable modes, so the accepted inner `ustar` remains required.
- The smallest consumer addition is `actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093` (v4.3.0). Its immutable action contract supports same-run retrieval by exact artifact ID into a named path without a token or another workflow/run lookup.
- `scripts/verify-built-identity.py` can inspect extracted Rust/JVM identities without compiling. The manifest verifier currently invokes `cargo metadata`; direct `Cargo.lock` derivation produces the same 214 Cargo component records at this commit, so eliminating that possible fresh-run dependency resolution is a narrow handoff dependency rather than a manifest redesign.
- Workflow choreography must keep the existing always-run diagnostic/evidence/cleanup path, but gate release-byte upload on overall producer success plus secret-scan success. A distinct required consumer job will `needs` the producer, download by artifact ID, and run no Cargo build, Gradle task, release builder, or mutable artifact lookup.

## Current evidence and untested boundaries

- `SOURCE INSPECTED`: required policy, campaign, predecessor invariants/completion packet, ledger, README/build metadata, workflow, inventory, release/manifest/identity/evidence/scan/walker owners, focused tests, and release/CI/install owner docs were reconciled.
- `UNIT TESTED`: the preexisting `tests/test_release_identity.py` suite passes 6 tests. `scripts/check-operations.py --probe artifact-provenance-pass` and `--probe ci-compose-retained` pass. `git diff --check` passes for the supplied inputs.
- Canonical archive implementation, archive mutation tests, pack/verify/extract integration, final local release construction, release upload, consumer job, final remote workflow, and post-run independent retrieval are `NOT RUN`.
- Disposable changed update, exact no-op, service restart, container restart, PostgreSQL restore, controlled recovery, protocol-client, real-player, and production observations are `NOT RUN` and deferred or forbidden here.
- No host, container, service, database, backup, snapshot, listener, route, public traffic, EULA record, tag, GitHub Release, player, or production state has been mutated.

## Next executable action

Implement and mutation-test the sole deterministic release archive/descriptor authority, including safe extraction and a dependency-free manifest component derivation; then connect it after two-root comparison before changing hosted artifact choreography.
