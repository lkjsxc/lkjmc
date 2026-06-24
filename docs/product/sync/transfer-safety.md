# Transfer safety

## Purpose

This document defines safe profile transfer between servers.

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
acknowledgements and recovery events. Proxy-side orchestration that waits on
source save acknowledgements before connecting remains pending.
