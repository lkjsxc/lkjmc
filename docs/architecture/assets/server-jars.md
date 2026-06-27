# Server jars

## Purpose

This target contract defines server jar assets for Velocity, Paper, Folia, and
Purpur.

## Sources

PaperMC download service remains the source for Paper, Folia, and Velocity
server jars. Purpur downloads use the Purpur project API and are treated as
Paper-compatible assets. The default channel is stable when the upstream source
exposes channels. User-facing release selection uses `--minecraft-release` when
available. Without an explicit release, lkjmc selects a Java 21-compatible
default for Paper-like and Folia backends.

## Kinds

Server jar kinds are `velocity`, `paper`, `folia`, `purpur`, `custom`,
`vanilla-custom`, and `modded-custom`. Folia is the default playable backend.
Purpur must never silently fall back to Paper; a missing Purpur asset reports an
exact provider error.

## Storage

Server jars are stored under the asset root in immutable paths such as:

```text
/opt/lkjmc/assets/server/papermc/folia/{sha12}-{fileName}
/opt/lkjmc/assets/server/papermc/velocity/{sha12}-{fileName}
/opt/lkjmc/assets/server/purpur/purpur/{sha12}-{fileName}
```

The database row records kind, platform, project, channel, name, file name,
path, SHA-256, size when known, source, and metadata.

## Install rule

Instances reference verified server assets. A playable apply blocks when the
required Velocity or Folia server jar cannot be downloaded and no verified local
asset already exists. Purpur templates require a verified Purpur asset.
