# Config

## Purpose

This document defines JSON runtime configuration contracts.

## Current main config

`lkjmc-core` parses and validates the main `/etc/lkjmc/lkjmc.json` shape with
root paths, database metadata, network defaults, jar registry, daemon HTTP,
asset policy, plugin policy, and local runtime settings. Validation rejects
relative product paths, empty names, invalid ports, invalid fallback ids, weak
asset User-Agents, and zero memory or stop timeout values.

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

Velocity sets `enabled=false` and `keepWarm=true`. The default Folia hub stays
warm until wake-and-join queuing exists. Non-entry backends may suspend when
empty after grace.

## Current instance config

Instance templates live under `templates/{template}.json` in the config root and
may define kind, memory, server port, command arguments, environment variables,
plugins, capabilities, and autosuspend policy. The daemon renders instance
directories from those templates.

## Playable additions

Playable bootstrap uses config sections for daemon HTTP, asset registry, plugin
policy, Java entry, Bedrock entry, forwarding secret file, and runtime adapter
settings. Product paths and secret files must be absolute.

```json
{
  "network": {
    "javaEntry": {
      "bindHost": "0.0.0.0",
      "port": 25565,
      "publicHosts": ["lkjsxc.com"],
      "preferredPublicHost": "lkjsxc.com"
    }
  }
}
```

`publicHosts` is optional. When present, entries must be non-empty hostnames and
`preferredPublicHost` must name one of them.

## Runtime adapter selection

`runtime.adapter` accepts `local-process` and, after Kubernetes config is
complete, `kubernetes`. Unknown adapter names fail JSON config parsing instead
of silently selecting local behavior. Daemon status and doctor report the active
adapter and capabilities.

## Kubernetes adapter config

Kubernetes config defines namespace, kubeconfig path or in-cluster mode, server
image reference, service type policy, storage class and size, readiness probes,
log limits, and CPU and memory requests. Missing required fields make the config
invalid.

## Web control config

The current private web control surface uses `daemonHttp.enabled`,
`daemonHttp.address`, and `daemonHttp.tokenFile`. Browser login accepts the same
operator token source, stores in-memory sessions tied to a safe token
fingerprint, and generates per-session CSRF values. Diagnostics print token-file
paths or fingerprints, never raw token bytes.

## Java schema mirror

The JVM common module validates the daemon HTTP URL, token source, instance id,
platform role, locale defaults, public host fields, and feature flags against a
Rust-owned schema artifact or drift-checked mirror.

## Current boundary

The daemon and installer load and write the current main JSON config. The daemon
`config.reload` command reloads the same config path used at startup and applies
database and root path changes to new operations. The Discord service owns a
separate JSON config for bot and interaction settings.
