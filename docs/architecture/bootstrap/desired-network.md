# Desired network

## Purpose

This contract defines the desired default network consumed by the
bootstrap planner.


## Status

implemented

## Topology

- Proxy instance: `proxy`, kind Velocity, Java TCP bind `0.0.0.0:25565`.
- Backend instance: `hub`, kind Folia, backend TCP bind `127.0.0.1:25566`
  unless container routing requires `0.0.0.0` inside Compose.
- Fallback server: `hub`.
- Default game mode: survival.
- Proxy memory: 512 MiB.
- Hub memory: 2048 MiB.

## Security posture

Velocity uses online mode and modern forwarding. Folia runs with
`online-mode=false` because the proxy authenticates players. The forwarding
secret is generated once, stored in a private file, and rendered into backend
proxy config without being printed.

## Backends

Folia is the default hosted backend. Paper remains supported. Purpur is a
Paper-compatible backend option and must be selected explicitly through config,
template, or jar choice. Purpur is not treated as Folia-compatible.

## Plugins

The integrated target requires verified `lkjmc` Paper and Velocity plugin
assets. Java protocol compatibility and Bedrock entry are optional policy-driven
features that may be withdrawn with diagnostics.
