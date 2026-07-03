# Transfer safety

## Purpose

This document defines safe profile transfer between servers.


## Status

implemented

## Flow

1. Proxy receives a transfer request.
2. Source plugin captures a snapshot on the correct player scheduler.
3. Source plugin writes the snapshot asynchronously with the active lease.
4. Source plugin acknowledges the saved revision.
5. Proxy connects the player to the target server.
6. Target plugin obtains the lease and applies the revision on the scheduler.
7. Target plugin records the active session.

## Crash rule

Uncertain saves and expired leases create recovery events instead of silently
overwriting inventory-like data.

## Current status

The current slice exposes daemon `player.transfer.saved` and
`player.recovery.report` commands that record audit-backed transfer
acknowledgements and recovery events. Velocity sends a `lkjmc:profile` plugin
message to the source Paper server, waits for the Paper adapter to persist a
snapshot, and only then connects the player to the target server. Cross-server
home and warp commands create PostgreSQL pending teleport records, request a
profile-safe proxy transfer, and the target Paper server consumes the pending
location on join. Temporary instance transfers create a daemon-validated
short-lived transfer intent before Velocity invokes the same profile-safe
bridge. Menu server rows emit the same `lkjmc:profile` transfer request, so menu
clicks save the source profile before Velocity connects the player. Cross-server
teleport requests are accepted through the proxy bridge, which saves the source
profile before connecting and sends the accepted target location to the
destination Paper server. If the source server does not acknowledge in time, the
transfer is denied instead of risking a stale target load.
