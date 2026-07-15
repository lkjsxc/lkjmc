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
version pin, so the repository does not invent one. The Gradle wrapper records
the checksum published beside its distribution, and the host Rust bootstrap
checksum is verified against its acquisition source. `rust-toolchain.toml`,
`Cargo.lock`, wrapper files, and base-image references are checksum inputs. An
unavailable digest, distribution checksum, bootstrap checksum, or locked
dependency fails closed; offline mode may use cached bytes only after the same
verification.

`scripts/check-operations.py --probe toolchain-acquisition-pass` verifies these
rules and its mutation suite removes each required pin and expects rejection.
Release acquisition never falls back to `latest`, a branch, an unchecked URL,
or an unverified local cache.

## Manifest and inventory

After building, run:

```sh
scripts/artifact-manifest.py --output artifact-manifest.json \
  target/release/lkjmc target/release/lkjmc-daemon \
  platforms/jvm/velocity/build/libs/lkjmc-velocity.jar
```

The private JSON inventory is tied to the exact clean commit. It records SHA-256,
size, kind, component, source path, and provenance for every supplied binary and
jar; pinned image identity; configuration contracts; lockfiles; and an SBOM-like
list of Rust and Gradle components. Missing expected release artifacts,
untracked inputs, duplicate destinations, secret-shaped names, the supplied
generated credential canary, and nonregular files fail the command. Embedded
credential-shaped redaction fixtures are inventory bytes, not secret evidence. The
manifest itself is checksummed.

## Reproducibility and publication

Build release binaries and jars twice from separate clean exports with the same
pinned toolchains and no shared output directory. Compare checksums and retain
both manifests. Byte differences are a release failure unless a reviewed,
recorded format-specific explanation and normalized comparison exists. Image
configuration digests and source inputs are always compared even when container
layer timestamps prevent a byte-identical image ID.

Publish checksums beside exactly those artifacts and verify from a separate
private directory with `sha256sum --check`. Checksums and commit identity prove
byte/source association, not publisher identity. Signing requires a separately
trusted key and verified signature; absence is an explicit external skip.
