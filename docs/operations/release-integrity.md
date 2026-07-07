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

## Verification

Operators verify downloaded artifacts with:

```sh
sha256sum --check SHA256SUMS
```

SBOM generation can be added beside this runbook, but checksum generation is the
minimum release integrity gate.
