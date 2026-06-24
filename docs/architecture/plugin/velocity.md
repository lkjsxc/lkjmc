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
`/hub`, plus MOTD and tab-list listeners. `/lkjmc status` reports proxy player
count behind the admin status permission. `/lkjmc server list` lists registered
Velocity servers behind the admin instance list permission. `/hub` connects
players to a registered `hub` server or returns a failure message. The MOTD listener renders
a fixed `lkjmc network` description, and post-login tab header/footer shows the
current proxy player count. Daemon-backed instance operations, dynamic server
registry mutation, reload, restart scheduling, and transfer sync coordination
are not implemented or registered yet.
