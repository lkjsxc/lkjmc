# Playable network quickstart

## Purpose

This target contract defines the clean local path to a playable Java Minecraft
network managed by `lkjmc`.

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
Paper, Folia, or Purpur backend. Without acceptance, bootstrap blocks and prints
the flag or environment variable needed to continue.

## Expected result

The playable run starts PostgreSQL, `lkjmc-daemon`, Velocity instance `proxy`,
and Folia instance `hub`. Java clients connect to the configured public host or
TCP `25565` on the proxy and land on `hub`. The `lkjmc` Velocity and Paper
plugin jars are copied from verified assets into managed plugin directories
before start.

Domain entry example:

```sh
LKJMC_PLAYABLE_PUBLIC_HOST=lkjsxc.com LKJMC_ACCEPT_MINECRAFT_EULA=1 \
  docker compose -f docker-compose.yml -f docker-compose.playable.yml \
  up --build playable
```

Status and final output should include `java: lkjsxc.com:25565`. Compose also
honors `LKJMC_PLAYABLE_JAVA_PORT`, `LKJMC_PLAYABLE_JAVA_BIND_HOST`, and
`LKJMC_PLAYABLE_BEDROCK_PORT` for config and published ports.

## Truthfulness rule

Bootstrap status may report success only after the daemon owns the Java
processes, the proxy status ping works, and required plugin jars are installed.
Optional Bedrock or compatibility plugins may be withdrawn with diagnostics.
