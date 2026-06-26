# Plugin jars

## Purpose

This target contract defines local and third-party plugin assets.

## Local plugins

Gradle `shadowJar` outputs for `lkjmc-paper` and `lkjmc-velocity` are copied
into the asset registry with SHA-256 and size before installation:

```text
/opt/lkjmc/assets/plugin/lkjmc/paper/{sha12}-lkjmc-paper.jar
/opt/lkjmc/assets/plugin/lkjmc/velocity/{sha12}-lkjmc-velocity.jar
```

Playable bootstrap blocks if these integrated plugin assets cannot be built or
registered.

## Third-party plugins

Known plugin IDs are `viaversion`, `viabackwards`, `geyser`, and `floodgate`.
Their assets are downloaded only from APIs that provide hashes and sizes.
Verified assets are installed under predictable names in managed instance plugin
directories.

## Immutable rule

Assets are never overwritten in place. A changed file creates a new content
addressed path and a new registry row.
