# Config

## Purpose

This document defines JSON runtime configuration contracts.

## Current main config

`lkjmc-core` parses and validates the main `/etc/lkjmc/lkjmc.json` shape with
these current sections:

- root paths and socket path
- database connection metadata
- network defaults
- jar registry settings
- local runtime settings

Validation rejects relative product paths, empty names, invalid ports, a
fallback server that is not lowercase kebab-case, a jar User-Agent that does not
identify `lkjmc`, and zero memory or stop timeout values.

## Current instance config

Instance templates live under `templates/{template}.json` in the config root and
may define kind, memory, server port, command arguments, environment variables,
and a plugin map. The daemon renders instance directories from those templates.

## Playable additions

Playable bootstrap uses config sections for daemon HTTP, asset
registry, plugin policy, Java entry, Bedrock entry, forwarding secret file, and
runtime port ranges. Product paths and secret files must be absolute.

Network fields include `defaultLocale`, `fallbackServer`, `onlineMode`,
`velocityForwarding`, `forwardingSecretFile`, `javaEntry`, and `bedrockEntry`.
Bedrock uses UDP; Java uses TCP. Their ports must be valid and distinct unless
Bedrock is disabled.

Java entry separates local bind from public display:

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

Target plugin modes are `enabled`, `disabled`, and `auto`. ViaBackwards requires
ViaVersion after planning. Floodgate requires Geyser after planning. The asset
User-Agent must contain `lkjmc` and a contact string.

## Current boundary

The daemon and installer load and write the current main JSON config. The daemon
`config.reload` command reloads the same config file path used at startup and
applies database and root path changes to new operations. No Java schema mirror
exists yet.
