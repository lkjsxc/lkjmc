# Paper and Folia plugin

## Purpose

This document defines the Paper and Folia adapter contract.

## Responsibilities

- Keep platform API calls at adapter edges.
- Use scheduler bridges for player, entity, and world mutation.
- Call daemon HTTP asynchronously for product state.
- Keep English and Japanese player-visible messages in lockstep.

## Scheduler rules

Database, filesystem, network, and process work must not block Minecraft
scheduler threads. Completion callbacks that touch game state must re-enter the
correct platform scheduler.

## Current status

The Paper module builds a real plugin jar, exposes `/lkjmc` as the public admin
root, connects to daemon HTTP when configured, and drives profile, claim,
moderation, mail, kit, vote, daily reward, announcement, and GUI behavior
through adapters. Inventory menus must render metadata-bearing items, reduce
clicks through the common pure menu core, execute effects without blocking
scheduler threads, and avoid inventory closes except the explicit close button.
Folia-specific scheduling rules remain part of the platform boundary.

## Playable target

The managed `hub` backend receives the `lkjmc` Paper plugin from the asset
registry before start. Its environment provides `LKJMC_INSTANCE_ID=hub`, daemon
HTTP URL, and daemon token file. The plugin must fail clearly if daemon HTTP is
required but not configured, and it must never log tokens.

## Proxy target

Paper backends behind Velocity modern forwarding use `online-mode=false`, keep
BungeeCord forwarding disabled, and configure Paper Velocity proxy settings with
the same secret and online-mode as the proxy.
