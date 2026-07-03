# Surface state

## Purpose

This file records shipped external surfaces.

## Status

implemented

## Java adapters

- Java 21 common, Velocity, and Paper modules share command, locale, menu,
  daemon HTTP, and transfer contracts.
- Velocity registers the command tree, restart forwarding, profile transfer
  bridge, moderation listener, and localized player messages.
- Paper registers command adapters, lifecycle profile load/save, menus, docs,
  claims, random teleport, shops, exchange, effects, and transfer listeners.
- Paper sends passive Action Bar frames about every four ticks from cached daemon
  snapshots, with local-session fallback when remote fields are unavailable.
- Paper random teleport commands support free overworld RTP and paid Nether or
  End profile confirmation; Nether and End portals remain cancelled without
  charging points.
- Paper admin server-create menus disable unstartable create plans with daemon
  diagnostic lore instead of a vague unavailable state.
- Velocity reports managed-server registration state to the daemon, and public
  server rows transfer only when `instance.list` reports joinability.
- Java daemon clients send HTTP `POST /command`; blank or root endpoints resolve
  to `/command`.

## Web, Discord, and Compose

- Authenticated web operator routes are available behind daemon auth, including
  authenticated `/web` operator pages.
- Discord service config uses JSON, verifies signed interactions, maps Discord
  principals, and delegates supported slash commands to the daemon.
- Compose has one file with `verify`, `playable`, and `discord` profiles.
- Safe example JSON exists for daemon and Discord configuration.

## Current limits

- Live Discord, Bedrock, and playable smokes skip unless their guard variables
  and external prerequisites are supplied.
- `kubernetes` selectable runtime checks are guarded by the Kubernetes live
  smoke.
- Existing child process working directories are not rewritten in place after a
  config reload; new operations use updated config and templates.
