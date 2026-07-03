# Plugin provisioning

## Purpose

This contract defines how plugin jars reach managed instance plugin
directories.


## Status

implemented

## Asset first

Every plugin install starts from a verified immutable asset. Local `lkjmc` plugin
jars are registered from Gradle `shadowJar` outputs. Third-party plugin jars are
registered only after source API hash and size verification.

## Target directories

```text
/var/lib/lkjmc/instances/proxy/plugins/
/var/lib/lkjmc/instances/hub/plugins/
```

Expected managed file names:

```text
lkjmc-velocity.jar
lkjmc-paper.jar
ViaVersion.jar
ViaBackwards.jar
Geyser-Velocity.jar
floodgate-velocity.jar
```

## Install effect

A `plugin.install` effect creates the plugin directory, copies from the asset
path, hashes the target, compares it with the asset hash, and records an
installation row. It must not copy into a running instance without planning a
restart.

## Platform guard

Velocity-only plugins are installed only on Velocity. Paper or Folia plugins are
installed only on plugin-capable backends. Vanilla or custom servers receive no
third-party plugins unless a future contract proves plugin support.
