# Velocity plugin

## Purpose

This document defines the Velocity adapter contract.


## Status

implemented

## Responsibilities

- Register real Velocity commands only after handlers exist.
- Call daemon HTTP asynchronously for daemon-backed operations.
- Register daemon-discovered servers with Velocity.
- Validate runtime config through Java common before registering live actions.
- Keep transfer sync and profile safety coordinated with Paper backends.

## Current status

The Velocity module builds a real plugin jar. On proxy initialization it
registers `/lkjmc` from the shared command tree, including status, doctor,
server lifecycle, send, temporary send, wake send, reload, restart warning,
`/hub`, MOTD and tab-list listeners, and daemon-backed ban checks when HTTP is
configured. Startup calls daemon `instance.list` and registers returned
localhost server ports. `/hub` saves the source profile, then reports the actual Velocity connection
result to the player; it does not report a requested connection as success.

## Playable target

The managed proxy instance receives the `lkjmc` Velocity plugin from the asset
registry before start. Its environment provides daemon HTTP URL and token file.
After bootstrap or temporary-instance runtime creates or changes instances,
Velocity dynamic server registration must refresh so `/hub` and managed
transfers see ready backends promptly. Profile-safe menu, hub, send, and TPA
transfers wait for the connection result; failed connections send failure rather
than target-arrival or success feedback. The registry must skip instances whose
daemon `instance.list` row marks proxy registration disabled and must unregister
servers it previously registered when they disappear or become cleanup-only.

## Forwarding target

The default proxy uses online mode and modern player information forwarding with
a private `forwarding.secret` file. It must not mix forwarding modes or install
ProtocolSupport for the playable default.
