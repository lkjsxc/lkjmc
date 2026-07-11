# Minecraft commands

## Purpose

This document defines the shipped in-game command surface and its containment
boundary.

## Status

implemented

## Shipped Paper/Folia commands

- `/menu` opens the local documentation menu and requires `lkjmc.user.menu`.
- `/docs [search <query>|path]` opens the bundled documentation browser and
  requires `lkjmc.user.docs`.

The hotbar token opens the same local documentation UI. These surfaces read
bundled documents only; they do not call the daemon, consult grants, mutate
product state, or require a daemon credential.

## Shipped Velocity behavior

Velocity provides MOTD and tab-list presentation only. It registers no `/lkjmc`,
`/hub`, transfer, or other command.

## Withdrawn surfaces

Paper/Folia and Velocity `/lkjmc` commands, completion, admin commands, player
commands, claim commands, transfer commands, daemon-backed menus, and
proxy-server registration are withdrawn pending trusted identity/session
attestation. They must not be registered, packaged, or represented as a fallback
for the CLI or daemon.

A future reintroduction needs a separate owner contract, authenticated player
identity and session attestation, daemon-final authorization, source registration
proof, and built-artifact inspection. Platform permissions, `op`, caller-shaped
actors, and cached grants are not identity proof.

## Verification

The Java containment checker inspects production/test sources, resources, and
built plugin jars for absent daemon clients, bridge classes, and withdrawn command
classes. Paper Gradle tests inspect retained local resources and Velocity Gradle
tests cover pure MOTD/tab-list text; neither invokes a Minecraft runtime.
