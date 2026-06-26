# Quickstart troubleshooting

## Purpose

This target contract defines concise operator diagnostics for playable
bootstrap.

## Blocking diagnostics

- Missing Minecraft EULA acceptance: rerun with `--accept-minecraft-eula` or
  `LKJMC_ACCEPT_MINECRAFT_EULA=1`.
- PostgreSQL unavailable: start PostgreSQL and rerun `lkjmc db status`.
- Server jar download failed and no verified cached asset exists: restore
  network access or import a verified asset.
- Unmanaged instance path exists: move the path aside or adopt it through a real
  managed import command when one exists.

## Degraded diagnostics

- ViaVersion or ViaBackwards unavailable: Java play continues without protocol
  compatibility plugins.
- Geyser or Floodgate unavailable: Java play continues with Bedrock withdrawn.
- UDP `19132` unavailable in auto mode: Bedrock is withdrawn and Java remains
  playable.

## Commands

```sh
/opt/lkjmc/bin/lkjmc bootstrap status --json
/opt/lkjmc/bin/lkjmc bootstrap doctor
/opt/lkjmc/bin/lkjmc instance logs proxy --lines 100
/opt/lkjmc/bin/lkjmc instance logs hub --lines 100
```

Status output must identify whether a failure blocked Java play or only
withdrew an optional feature.
