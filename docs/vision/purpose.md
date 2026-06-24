# Purpose

## Purpose

This document states why `lkjmc` exists.

## Product intent

`lkjmc` should make a Minecraft network installable, observable, and operable
from a shell and from Minecraft itself. It centralizes desired state,
PostgreSQL-backed player data, process orchestration, jar management, and
localized in-game operations.

## Design center

One install command, one canonical data store, one daemon, one CLI, one Velocity
plugin, one Paper/Folia plugin, and shared JSON contracts.
