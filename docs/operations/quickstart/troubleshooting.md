# Quickstart troubleshooting

## Purpose

This contract defines concise operator diagnostics for playable
bootstrap.


## Status

implemented

## Blocking diagnostics

- Missing Minecraft EULA policy: on an operator-owned host, run `lkjmc-ops eula
  policy create` with the canonical config and policy paths, then rerun the
  systemd start. Do not add a request flag or environment-variable override.
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
- No SRV record for a hostname on port `25565`: hostname entry may still be
  correct when A or AAAA points at the proxy.
- Hostname and direct IP differ: run `lkjmc network diagnose HOST --port PORT`
  and compare DNS, TCP, and status ping steps.

## Commands

```sh
/opt/lkjmc/releases/current/bin/lkjmc bootstrap status --json
/opt/lkjmc/releases/current/bin/lkjmc bootstrap doctor
/opt/lkjmc/releases/current/bin/lkjmc instance logs edge-gateway --lines 100
/opt/lkjmc/releases/current/bin/lkjmc instance logs quartz-world --lines 100
```

The two instance IDs above are examples only. Status output must identify whether a failure blocked Java play or only
withdrew an optional feature. Domain diagnostics must explain whether DNS, SRV,
TCP reachability, Velocity bind, status ping, host routing, or backend state is
the next fix point.
