# Download policy

## Purpose

This target contract defines how `lkjmc` downloads external artifacts.

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

Hash verification is required for every downloaded artifact. Download adapters
write to a temporary file under the asset root, check expected size when known,
verify source checksum when available, compute SHA-256, fsync when practical,
and atomically rename to the content-addressed path. The daemon records failed
downloads in `asset_downloads` without secrets.

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
