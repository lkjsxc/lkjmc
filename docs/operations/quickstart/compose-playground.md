# Compose playground

## Purpose

This document defines the development Compose path for a playable network.


## Status

implemented

## Wrapper

The friendly wrapper requires explicit EULA acceptance:

```sh
./scripts/dev-up.sh --accept-minecraft-eula
```

Optional Bedrock policy flags:

```sh
./scripts/dev-up.sh --accept-minecraft-eula --bedrock-enabled
./scripts/dev-up.sh --accept-minecraft-eula --bedrock-disabled
```

## Direct Compose

```sh
LKJMC_ACCEPT_MINECRAFT_EULA=1 \
  docker compose --profile playable \
  up --build playable
```

The `playable` service uses PostgreSQL from the base Compose file and lets the
daemon start child Velocity and Paper processes through the local process
runtime. TCP `25565` and UDP `19132` are published by Compose, but Bedrock is
only marked enabled when its assets and UDP listener are verified.

## Volumes

The service persists config, data, logs, jars, and assets in named volumes so
repeated runs converge instead of downloading or generating new immutable assets
unnecessarily.
