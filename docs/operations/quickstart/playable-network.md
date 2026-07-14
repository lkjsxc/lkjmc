# Playable network quickstart

## Purpose

This contract defines the clean local path to a playable Java Minecraft
network managed by `lkjmc`.


## Status

implemented

## One bootstrap path

The operator edits only `/etc/lkjmc/lkjmc.json`; its `network` object is parsed
by `lkjmc-core`. Host install and Compose prepare dependencies, then both invoke
the daemon `bootstrap.plan`/`bootstrap.apply` path. No script launches Java,
renders network files, or applies Kubernetes manifests independently.

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
  docker compose --profile playable \
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
  docker compose --profile playable \
  up --build playable
```

Status and final output should include `java: lkjsxc.com:25565`. Compose also
honors `LKJMC_PLAYABLE_JAVA_PORT`, `LKJMC_PLAYABLE_JAVA_BIND_HOST`, and
`LKJMC_PLAYABLE_BEDROCK_PORT` for config and published ports.

## Inspection and recovery

Run `lkjmc bootstrap plan --json` before apply. It reports exact ordered
changes, no-op, or unsupported capabilities without effects. Status includes
the durable intent revision, request correlation, current apply outcome, and
failed step. Reapply repairs observed partial state; a converged reapply is a
no-op.

## Truthfulness rule

Bootstrap status may report success only after the daemon owns the Java
processes, verifies declared assets and listeners, and the proxy status ping
works. Kubernetes apply is unsupported unless mounted config, secret, and asset
capabilities are all declared and the adapter verifies them. Optional Bedrock
or compatibility assets may be withdrawn with diagnostics.
