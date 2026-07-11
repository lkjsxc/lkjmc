# Admin RBAC runtime

## Purpose

This document defines target daemon and CLI authorization architecture.


## Status

implemented

## Current status

The product role catalog, durable grants, grant/revoke/inspect daemon commands,
CLI management, admin audit rows, and daemon authorization checks for documented
admin command families are shipped. Java grant snapshots and `/lkjmc` visibility
are withdrawn pending trusted identity/session attestation.

## Principals

- Minecraft player UUID.
- Local CLI operator.
- System daemon.
- Local CLI operator.

Paper and Velocity services are not daemon principals while their daemon adapters
are withdrawn.

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

Paper and Velocity do not fetch or retain admin grant snapshots and do not
register daemon-backed admin commands or menus. A future adapter must present
trusted identity/session attestation before it can request visibility data;
cached grant data alone is never authorization proof.
