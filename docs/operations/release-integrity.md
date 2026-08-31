# Release integrity

## Purpose

Define the exact immutable release closure and the distinction between source, built, retained,
retrieved, verified, installed, and running identity.

## Canonical inventory

`config/release-artifacts.json` independently owns the shipped file set. The current closure is
exactly:

| Release member | Installed destination | Kind |
| --- | --- | --- |
| `source/lkjmc` | `bin/lkjmc` | native Rust executable |
| `source/lkjmc-daemon` | `bin/lkjmc-daemon` | native Rust executable |
| `source/lkjmc-ops` | `bin/lkjmc-ops` | native Rust executable |
| `source/lkjmc-common.jar` | `jars/lkjmc-common.jar` | Java jar |
| `source/lkjmc-paper.jar` | `jars/lkjmc-paper.jar` | Java jar |
| `source/lkjmc-velocity.jar` | `jars/lkjmc-velocity.jar` | Java jar |
| `source/lkjmc-daemon.service` | `share/lkjmc-daemon.service` | declarative systemd unit |
| `source/lkjmc-deployment-fence.conf` | `share/lkjmc-deployment-fence.conf` | declarative drop-in |

The manifest, archive builder, installed-layout verifier, and systemd unit must agree with this
eight-member set. Native members must be ELF binaries, not shebang programs. No release member or
unit command may require Python, a POSIX shell, Bash, or a deleted compatibility executable.
Configuration, credentials, server jars, worlds, logs, database dumps, and host policy remain
outside immutable release bytes.

## Build identity

`Cargo.toml` is the canonical version and license owner. Release construction requires a clean,
exact Git commit and uses a detached worktree, fresh Rust and Gradle outputs, and a per-build nonce;
ambient `target/`, Gradle outputs, ignored files, and unpublished parent objects are not inputs.
Rust binaries and Java manifests/descriptors expose the same commit, version, license, and clean
state. `lkjmc-ops --version` identifies the packaged operations authority.

The pinned verifier image acquires Rust, Java, Gradle, base images, and dependencies through their
committed checksums and lock files. The repository still has non-shipped Python and shell build and
verification helpers; they are development-language debt, not members of the release or installed
runtime authority.

## Manifest and archive

The generated manifest records every release member's path, type, mode, size, SHA-256, provenance,
source commit, product version, contracts, images, and component inventory. Its sidecar binds the
manifest bytes. Verification derives the expected paths from the independent release inventory and
rejects missing, extra, traversing, linked, nonregular, wrong-mode, or digest-mismatched members.

The canonical handoff is a deterministic uncompressed POSIX `ustar` plus its checksum sidecar and
`release-handoff.json`. Archive verification parses raw headers, extracts through a no-follow
private staging tree, checks the complete manifest and embedded identities, and removes the
temporary result by retained identity. The GitHub artifact-service digest, archive digest, manifest
digest, retention period, and installed identity are separate facts.

## Reproducibility and consumption

Build the release twice from separate fresh exports in the same pinned environment and compare the
complete path/type/mode/size/digest closure. An unexplained difference fails the release. A
host-native output is a different environment and is not byte-reproducibility evidence for the
pinned verifier.

An operator must independently obtain the manifest SHA-256 and pass it to the exact
`bin/lkjmc-ops` contained by that anchored target release. Download, successful extraction, or a
matching mutable label does not authorize installation or prove an update. `lkjmc-ops release
verify` is read-only; update/recovery acceptance adds the live fleet, PostgreSQL, fence, systemd,
and readiness boundaries documented in [install.md](install.md).

Secret scanning covers the source context, release members, image layers, and bounded handoff
evidence. Checksums bind bytes but do not authenticate a publisher; signing remains a separate
future decision.
