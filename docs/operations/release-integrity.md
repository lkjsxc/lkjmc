# Release integrity

## Purpose

This runbook defines integrity checks for release artifacts and plugin jars.

## Status

implemented

## Checksums

Produce SHA-256 checksums for built artifacts before publishing:

```sh
scripts/release-checksums.sh target/release/lkjmc-daemon build/libs/*.jar \
  > SHA256SUMS
```

Do not include token files, databases, logs, Minecraft worlds, or `tmp/` paths
in release checksum inputs. Artifact names should include component, platform,
and version chosen by the release process.

## Release procedure

1. Start from a clean source checkout and record `git rev-parse HEAD` as the
   source commit identity before building. Do not substitute a branch name.
2. Build the exact daemon and plugin artifacts intended for publication.
3. Run the applicable verification tier before checksumming them.
4. Generate and publish `SHA256SUMS` beside those exact files.
5. Download the published files into a separate directory and verify the manifest
   before installation.

```sh
sha256sum --check SHA256SUMS
```

A matching checksum proves bytes match the manifest; it does not prove artifact
provenance, signing identity, compatibility, or that live smokes ran. Record the
artifact names, manifest, verification command output, and verification-tier
result and source commit identity in the release evidence. SBOM generation can
be added beside this runbook, but it is not an implemented release gate.
