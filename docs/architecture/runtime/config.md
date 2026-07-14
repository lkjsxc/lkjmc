# Config

## Purpose

This document defines JSON runtime configuration contracts.


## Status

implemented

## Current main config

`lkjmc-core` parses and validates the main `/etc/lkjmc/lkjmc.json` shape with
root paths, database metadata, database pool size, one network intent, jar
registry, daemon HTTP, and runtime settings. The closed `network` object owns
instances, routes, listeners, authentication, forwarding, immutable assets, and
required adapter capabilities. There is no second instance topology, host launch
file, or declarative compiler input.

Validation rejects relative product paths, unknown members, invalid or duplicate
ids and ports, dangling references, ambiguous ownership, non-SHA-256 assets,
weak User-Agents, invalid database pool sizes, and zero or excessive memory,
counts, deadlines, or timeouts.

## Database

`database.poolSize` is optional, defaults to `8`, and must be between `1` and
`64`. The daemon builds one PostgreSQL pool from the configured database URL and
pool size during startup.

## Server implementation choices

Managed server implementation values are `velocity`, `paper`, `folia`,
`purpur`, `vanilla-custom`, and `modded-custom`. Folia is the default playable
backend. Purpur is Paper-compatible and uses the Paper plugin path unless a real
Purpur-specific adapter is documented.

Runtime capabilities are explicit data, for example:

```json
{
  "implementation": "folia",
  "capabilities": {
    "regionScheduler": true,
    "paperApi": true,
    "purpurConfig": false,
    "velocityForwarding": true
  }
}
```

Templates use capabilities for rendering. Plugin code must not randomly branch
on implementation strings.

## Autosuspend policy

Network defaults, templates, and instances may define:

```json
{
  "autosuspend": {
    "enabled": true,
    "idleGraceSeconds": 300,
    "minimumUptimeSeconds": 120,
    "heartbeatStaleSeconds": 90,
    "emptyHeartbeatCount": 2,
    "stopWhenEmpty": true,
    "deleteWhenExpired": false,
    "keepWarm": false
  }
}
```

Velocity sets `enabled=false` and `keepWarm=true`. These values remain parsed
desired configuration only: no reconciler or command currently starts, stops,
wakes, or queues a backend from autosuspend data.

## Current instance config

Instance templates live under `templates/{template}.json` in the config root and
may define kind, memory, server port, command arguments, environment variables,
plugins, capabilities, and autosuspend policy. They are parsed input only while
external rendering and launch effects remain denied.

## Playable intent

The production-parsed example is `config/defaults/daemon.json.example`.
`listeners` owns bind/public addresses, `routes` owns fallback order, `auth`
owns proxy authentication, and `forwarding.secretFile` owns the absolute secret
path. A public listener requires an explicit wildcard bind. Required asset
digests are immutable declarations, not download success claims.

## Runtime adapter selection

`runtime.adapter` accepts `local-process` and, after Kubernetes config is
complete, `kubernetes`. Unknown adapter names fail JSON config parsing instead
of silently selecting local behavior. Status reports the selected adapter, but
command lifecycle denies every external adapter capability.

## Kubernetes adapter config

Kubernetes config defines namespace, kubeconfig path or in-cluster mode, server
image reference, service type policy, storage class and size, readiness probes,
log limits, and CPU and memory requests. Missing required fields make the config
invalid.

## Web control config

The private web control surface uses `daemonHttp.enabled`,
`daemonHttp.address`, and `daemonHttp.tokenFile`. `enabled=false` starts no TCP
listener. After config defaults and every CLI override have been applied, an
enabled listener must be exactly `127.0.0.1:PORT`, with a nonzero port.
Hostnames, every other `127/8` address, wildcard and unspecified addresses,
IPv6, and IPv4-mapped IPv6 forms are rejected. Browser login accepts the same operator token source,
stores bounded session and credential fingerprints only, renews the cookie with
server expiry, and derives per-session CSRF values. Diagnostics print token-file
paths or fingerprints, never raw token bytes.

## Java boundary

Local-safe Java plugins receive no daemon HTTP URL, token source, instance role,
or daemon feature flag. The former JVM schema mirror is withdrawn with daemon
adapters pending trusted identity/session attestation.

## Verification

`scripts/check-config-examples.py` invokes the production Rust CLI parser for
every JSON example and confirms that an invalid bounded field is rejected. It is
not a duplicate Python schema or a synthetic config implementation.

## Current boundary

The daemon and installer load and write the current main JSON config. Every
field is restart-required: `config.reload` returns non-success
`config.restart_required` and neither reads nor applies the file. The Discord
service owns a separate JSON config for bot and interaction settings.
