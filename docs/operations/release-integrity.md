# Release integrity

## Purpose

Define commit-tied artifact inventory and immutable acquisition checks.

## Status

implemented

## Acquisition

`Dockerfile` names every base image by a digest verified against the registry.
Rust comes from that pinned image rather than an unverified installer pipe. Apt
packages come from signed repository metadata and the build verifies the
acquired package inventory; source repositories do not provide a durable exact
version pin, so the repository does not invent one. The custom Gradle launcher parses `distributionUrl` and
`distributionSha256Sum` from the wrapper properties. It rejects a missing,
zero, or malformed checksum, downloads into a private temporary file, verifies
SHA-256 before unzip or atomic cache publication, and verifies cached bytes and
the extracted closure before execution. A corrupt cache fails closed rather
than being trusted or silently repaired. The host Rust bootstrap checksum is
verified against its acquisition source. `rust-toolchain.toml`, `Cargo.lock`,
wrapper files, and base-image references are checksum inputs. An unavailable
digest, checksum, or locked dependency fails closed; offline mode may use
cached bytes only after the same verification.

`scripts/check-operations.py --probe toolchain-acquisition-pass` verifies these
rules and its mutation suite removes each required pin and expects rejection.
Release acquisition never falls back to `latest`, a branch, an unchecked URL,
or an unverified local cache.

## Build identity

`Cargo.toml` under `[workspace.package]` is the canonical product version and
license source for Rust and Gradle. The current pre-release is
`0.1.0-alpha.1`, licensed under Apache-2.0 to match the root `LICENSE`. JVM
artifact names are version-independent; Paper and Velocity descriptors, JAR
manifests, and generated JVM build constants carry the canonical version.

Ordinary Rust and JVM builds record an observed Git `HEAD` when available but
report dirty state as `unknown`; a gitless developer build reports both values
as `unknown`. This avoids turning Cargo's warm build-script cache into a false
clean-worktree claim.

A release build is stricter. `scripts/build-release.sh` requires a clean Git
checkout, creates a detached worktree at that exact object, generates a new
`LKJMC_BUILD_NONCE`, and builds every artifact in that fresh worktree to a
private output outside the source checkout. Rust and Gradle accept
`LKJMC_SOURCE_COMMIT` only with that nonce, a matching Git `HEAD`, and an
immediately observed clean tracked and nonignored-untracked closure. A supplied
commit in a gitless tree is rejected. `scripts/create-source-git-bundle.sh`
requires a clean, non-shallow checkout at that exact commit, exports only the
explicit `refs/bundles/lkjmc-source` ref, and proves the bundle through an empty
repository import and detached clean checkout before publication. Exported CI
source accepts only that one advertised ref, imports it into an empty repository,
checks full object connectivity, and compares the exported tracked closure to
the object before release construction. The verifier image checks required
entrypoints for executable modes but does not rewrite tracked source modes after
the export is copied. Ignored files are excluded from release inputs because
construction occurs in a fresh detached worktree. The release
output parent must not be group- or other-writable; its owner is part of the
trusted local build boundary. Failure cleanup removes the newly created output
only while its device and inode still match.

Operators can inspect the identity with `lkjmc version`, `lkjmc --version`, or
`lkjmc --json version`. Daemon status and health responses include the same
build object, daemon and Discord binaries support `--version`, and both JVM
plugins log their generated identity during startup. `scripts/build-release.sh`
rejects binaries, manifests, plugin descriptors, or generated JVM constants
that do not expose the exact release commit, version, license, and clean state.
It does not consume ambient `target/` or Gradle outputs from the caller.

## Manifest and inventory

`config/release-artifacts.json` is the independently authored release closure.
`scripts/build-release.sh` freshly builds the Rust binaries and shaded JARs and
copies those outputs plus the tracked deployer, artifact publisher,
backup/restore tools, restart helper, and canonical systemd unit into an
otherwise empty private release source directory. Static operational files come
from the same detached clean Git object, not the caller's ambient checkout.
`scripts/artifact-manifest.py` derives expected paths from that contract, not
from manifest contents, and requires exact set equality. It also derives all tracked `config/` and
`contracts/` files, toolchain and package/build manifests, and pinned image
identities independently and requires exact equality on verification.

The private JSON inventory is tied to the exact clean commit. It records SHA-256,
size, kind, destination, provenance, contracts, images, and an SBOM-like list of
Rust and Gradle components. Missing, extra, duplicate, traversing, symlinked, or
nonregular entries fail. The adjacent sidecar binds the manifest bytes and is
strictly parsed. To avoid recursion, the manifest does not inventory itself or
its sidecar; the sidecar binds the manifest, and the retained-evidence index
must include both files. `scripts/verify-artifact-manifest.py` checks that full
closure before install, scan, or publication. Executable manifest mutation
checks create and commit a private temporary Git fixture, so production
inventory code has no synthetic identity mode. Real release and lab paths
attach and verify the exported Git object.

## Reproducibility and publication

Build release binaries and jars twice from separate clean exports with the same
pinned toolchains and no shared output directory. Compare checksums and retain
both manifests. Byte differences are a release failure unless a reviewed,
recorded format-specific explanation and normalized comparison exists. Image
configuration digests and source inputs are always compared even when container
layer timestamps prevent a byte-identical image ID.

Publish checksums beside exactly those artifacts and verify from a separate
private directory. Before publication, `scripts/scan-secrets.py` scans every
release byte, the complete build context, every saved image layer, and bounded
retained evidence for a generated random canary and credential values. Canary
matching covers arbitrary bytes; credential patterns require printable URL
fields or a canonical `Bearer` header boundary so adjacent binary string-table
markers are not fabricated into a credential. Safe parameter names such as
`password`, `tokenFile`, and `databaseUrl` without values are not findings.
Checksums and commit identity prove byte/source association, not publisher
identity. Signing requires a separately trusted key and verified signature;
absence is an explicit external skip.

The system updater therefore requires the operator to supply the release
manifest SHA-256 separately from the extracted release directory. The packaged
publisher verifies that anchor, the strict sidecar, every source byte, and the
exact artifact set before mutation. A sidecar transferred beside a modified
manifest is not an independent trust anchor. Installed releases retain the
manifest and sidecar under `meta/` so later updates can verify the root-owned
current tree before stopping the service.
