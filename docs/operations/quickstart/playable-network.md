# Playable network quickstart

## Purpose

This target contract defines the desired clean local path to a playable Java
Minecraft network managed by `lkjmc`.

## Target commands

Host install target:

```sh
sudo ./scripts/install.sh --playable --accept-minecraft-eula
/opt/lkjmc/bin/lkjmc bootstrap status
```

Development Compose target:

```sh
./scripts/dev-up.sh --accept-minecraft-eula
```

Equivalent direct Compose target:

```sh
LKJMC_ACCEPT_MINECRAFT_EULA=1 \
  docker compose -f docker-compose.yml -f docker-compose.playable.yml \
  up --build playable
```

## EULA rule

The operator must pass `--accept-minecraft-eula` or set
`LKJMC_ACCEPT_MINECRAFT_EULA=1` before the daemon writes `eula.txt` or starts a
Paper or Folia backend. Without acceptance, bootstrap blocks and prints the flag
or environment variable needed to continue.

## Expected result

The target playable run starts PostgreSQL, `lkjmc-daemon`, Velocity instance
`proxy`, and Paper instance `hub`. Java clients connect to TCP `25565` on the
proxy and land on `hub`. The `lkjmc` Velocity and Paper plugin jars are copied
from verified assets into the managed instance plugin directories before start.

## Truthfulness rule

Bootstrap status may report success only after the daemon owns the Java
processes, the proxy status ping works, and required plugin jars are installed.
Optional Bedrock or compatibility plugins may be withdrawn with diagnostics.
