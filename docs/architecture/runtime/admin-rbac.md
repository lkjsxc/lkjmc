# Admin RBAC runtime

## Purpose

This document defines target daemon and CLI authorization architecture.

## Current status

The product role catalog, durable grants, grant/revoke/inspect daemon commands,
CLI management, admin audit rows, daemon authorization checks for documented
admin command families, and adapter-side grant snapshot caches for shared
`/lkjmc` visibility are shipped.

## Principals

- Minecraft player UUID.
- Local CLI operator.
- System daemon.
- Paper plugin service.
- Velocity plugin service.

## Enforcement

HTTP service tokens authenticate plugins, not end users. Privileged daemon
commands must receive an actor, optional subject, command name, request body,
and correlation id. Authorization resolves the subject or local CLI principal
against durable grants before mutation. Denials return structured safe errors
and write audit rows.

## CLI

The local CLI should own first-owner bootstrap, role listing, grant, revoke,
inspect, and audit tail operations. Bootstrap requires local host access, records
audit, and does not print generated secrets.

## Adapters

Paper and Velocity keep asynchronous admin grant snapshots for synchronous
`/lkjmc` visibility, completion, and admin menu enabled states. Snapshots are
fetched on join and refreshed without blocking scheduler threads. Snapshot
staleness can affect visibility but must not bypass daemon enforcement.
