# Playable default

## Purpose

This target contract defines the default user-visible playable network.

## Topology

- `proxy`: Velocity public Java entry bound to TCP `0.0.0.0:25565` by default.
- `hub`: Folia survival backend on TCP `127.0.0.1:25566`, or `0.0.0.0` inside
  Compose when only the proxy port is published.
- Fallback server: `hub`.
- MOTD: `lkjmc network`.
- Default player locale: English, with Japanese kept in lockstep.

## Forwarding

Velocity uses online mode, secure key authentication, and modern player
information forwarding. Folia uses `online-mode=false`, disabled BungeeCord
forwarding, and a Paper-compatible Velocity proxy config whose secret matches
Velocity's `forwarding.secret` file.

## Backends

Paper and Purpur are supported Paper-compatible backend choices. Folia remains
separate because region scheduling changes plugin execution rules. The same
`lkjmc` Paper plugin jar runs on Paper-like backends unless a real specialized
adapter is documented.

## Secrets

Daemon HTTP listens on `127.0.0.1:8765` and reads a bearer token from an
unreadable token file. The forwarding secret is generated with at least 32 bytes
of entropy, stored privately, reused idempotently, and never printed.

## Bedrock entry

Optional Bedrock status references UDP `19132`. The Java network remains
playable when Bedrock is withdrawn in auto mode.

## Status expectations

`lkjmc bootstrap status --json` reports `proxy` and `hub` states, ports,
installed `lkjmc` plugin state, withdrawn optional plugin reasons, configured
public hosts, effective Java target, and the next connection command. Success
requires daemon-owned Java processes and a valid Java status ping through
Velocity.
