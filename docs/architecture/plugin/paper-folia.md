# Paper and Folia plugin

## Purpose

This document defines the target server plugin behavior.

## Responsibilities

- Provide a Folia-safe scheduler bridge.
- Capture and apply player profile snapshots.
- Provide inventory UI and localized player commands.
- Send server heartbeats.
- Run database and daemon operations asynchronously.

## Scheduler rules

Entity mutations run on player or entity schedulers. Region mutations run on
region schedulers. Database, filesystem, network, and process operations never
block scheduler threads.

## Current status

The first Paper/Folia slice builds a real plugin jar. The plugin lifecycle
registers `/lkjmc status` and `/menu`, creates a Folia-aware scheduler bridge,
loads Java common localization resources, opens a localized root inventory menu,
and cancels tracked scheduled work on disable. The plugin descriptor declares
Folia support for this limited scheduler-bridge-backed surface. Profile
snapshot capture/apply, server heartbeat, teleport, homes, warps, points,
achievements, HUD, and daemon-backed instance operations are later slices and
are not registered yet.
