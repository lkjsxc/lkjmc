# Server jars

## Purpose

This target contract defines server jar assets for Velocity, Paper, and Folia.

## Sources

PaperMC download service remains the source for Paper, Folia, and Velocity
server jars. The default channel is stable. User-facing release selection uses
`--minecraft-release`. Without an explicit Paper or Folia release, lkjmc selects
a Java 21-compatible default instead of the newest upstream release.

## Storage

Server jars are stored under the asset root in immutable paths such as:

```text
/opt/lkjmc/assets/server/papermc/paper/{sha12}-{fileName}
/opt/lkjmc/assets/server/papermc/velocity/{sha12}-{fileName}
```

The database row records kind, platform, project, channel, name, file name,
path, SHA-256, size, source, and metadata.

## Install rule

Instances reference verified server assets. A playable apply blocks when the
required Velocity or Paper server jar cannot be downloaded and no verified local
asset already exists.
