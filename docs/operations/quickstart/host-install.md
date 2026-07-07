# Host install quickstart

## Purpose

This contract defines the host installer behavior for a playable network
on Ubuntu-like LXC and WSL2 hosts.


## Status

implemented

## Daemon-only mode

Without `--playable`, `scripts/install.sh` may keep the current daemon install
path: packages, PostgreSQL, service user, roots, JSON config, Rust binaries,
migrations, daemon start, and `lkjmc doctor`. `config/defaults/daemon.json.example`
uses the current `LkjmcConfig` JSON shape and is validated by fast checks for
hand-authored installs. Required roots, socket, database, network, jars, HTTP,
assets, plugins, and runtime fields are present; optional defaults may still be
kept by operators when they create their final file.

## Playable mode

With `--playable`, the installer must start the daemon and then ask the daemon
to run playable bootstrap:

```sh
sudo ./scripts/install.sh --playable --accept-minecraft-eula
```

Supported target flags:

```text
--playable
--accept-minecraft-eula
--bedrock auto|enabled|disabled
--java-bind-host HOST
--java-port PORT
--java-public-host HOST
--bedrock-port PORT
--no-start
```

## Secrets

The installer must generate or reuse secret files with restrictive permissions:

- `/etc/lkjmc/database.secret`
- `/etc/lkjmc/daemon-http.token`
- `/etc/lkjmc/forwarding.secret`

No generated secret may be printed or placed on a daemon command line.

## Final output

Playable mode prints compact connection information: Java address, Bedrock
state, proxy state, hub state, status command, and a log command. When
`--java-public-host lkjsxc.com` is supplied, the Java line prints
`java: lkjsxc.com:25565`. It must not claim success before bootstrap has
completed real effects.
