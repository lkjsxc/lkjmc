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

## Manifest and inventory

`config/release-artifacts.json` is the independently authored release closure.
`scripts/build-release.sh` copies every declared binary and shaded JAR into an
otherwise empty private release source directory. `scripts/artifact-manifest.py`
derives expected paths from that contract, not from manifest contents, and
requires exact set equality. It also derives all tracked `config/` and
`contracts/` files, toolchain and package/build manifests, and pinned image
identities independently and requires exact equality on verification.

The private JSON inventory is tied to the exact clean commit. It records SHA-256,
size, kind, destination, provenance, contracts, images, and an SBOM-like list of
Rust and Gradle components. Missing, extra, duplicate, traversing, symlinked, or
nonregular entries fail. The adjacent sidecar binds the manifest bytes and is
strictly parsed. To avoid recursion, the manifest does not inventory itself or
its sidecar; the sidecar binds the manifest, and the retained-evidence index
must include both files. `scripts/verify-artifact-manifest.py` checks that full
closure before install, scan, or publication.

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
retained evidence for a generated random canary and credential values. Safe
parameter names such as `password`, `tokenFile`, and `databaseUrl` without
values are not findings. Checksums and commit identity prove byte/source
association, not publisher identity. Signing requires a separately
trusted key and verified signature; absence is an explicit external skip.
