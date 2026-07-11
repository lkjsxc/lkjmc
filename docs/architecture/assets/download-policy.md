# Download policy

## Purpose

This contract defines how `lkjmc` downloads external artifacts.


## Status

implemented

## Sources

- PaperMC API: Paper, Folia, and Velocity server jars.
- Purpur API: Purpur server jars.
- Modrinth API: ViaVersion and ViaBackwards plugin jars.
- GeyserMC API: Geyser and Floodgate plugin jars.

HTML scraping is not allowed. Explicit URLs are allowed only when paired with a
trusted hash and size when the source provides size.

## User agent

Every external request uses a configured User-Agent that contains `lkjmc` and a
contact string. The default is:

```text
lkjmc (+https://github.com/lkjsxc/lkjmc)
```

## Verification

Hash verification is required for every downloaded artifact. One shared downloader
holds a bounded per-target lock, streams into one same-directory temporary file,
and computes MD5, SHA-256, and SHA-512 with the byte count in that single pass.
It checks the supplied size and source checksum, fsyncs the temporary file, then
atomically renames it to the content-addressed target. A failed, truncated, or
concurrent attempt leaves no partial final file and removes its temporary file.
Before returning every Paper, Folia, Velocity, or Purpur server-transfer failure,
the daemon writes an `asset_downloads` row with `failed`, no asset link, and no
claimed success. The row keeps project, channel, expected size, and trusted
SHA-256 when supplied, but a URL has no user info, query, or fragment and the
error is generic. An audit-write error is also a download failure.

## Retry and lock behavior

Retries may be used for transient network failures, but a command succeeds only
when a verified asset row and file exist. Catalog or lockfile metadata records
which source, project, channel, hash, size, and file name selected an asset.

## Failure modes

- Required server jar failure blocks playable bootstrap.
- Required `lkjmc` plugin build failure blocks integrated playable bootstrap.
- ViaVersion or ViaBackwards verification failure withdraws Java compatibility in
  auto mode.
- Geyser or Floodgate verification failure withdraws Bedrock in auto mode.
- Purpur provider failures report Purpur-specific errors and never fall back to
  Paper.
