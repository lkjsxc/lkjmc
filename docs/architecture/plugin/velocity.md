# Velocity plugin

## Purpose

This document defines the target proxy behavior.

## Responsibilities

- Initialize after Velocity initialization.
- Check daemon and database connectivity.
- Observe desired server registry.
- Register dynamic servers.
- Provide `/hub` and functional `/lkjmc` admin commands.
- Render MOTD and tab list.
- Coordinate profile-safe transfers.
- Route to fallback servers when targets are unavailable.

## Current status

The Velocity module builds a real Velocity plugin jar with an annotated
composition root. On proxy initialization it registers `/lkjmc status` and
`/hub`. `/lkjmc status` reports proxy player count behind the admin status
permission. `/hub` connects players to a registered `hub` server or returns a
failure message. Daemon-backed instance operations, dynamic server registry,
MOTD event wiring, tab list scheduling, restart scheduling, and transfer sync
coordination are not implemented or registered yet.
