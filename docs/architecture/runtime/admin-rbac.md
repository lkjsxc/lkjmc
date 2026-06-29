# Admin RBAC runtime

## Purpose

This document defines target daemon and CLI authorization architecture.

## Current status

The product role catalog is shipped as pure Rust data and is visible through
`admin.role.list`. Daemon-enforced grants are not shipped yet, so existing
privileged paths still rely on platform permissions or local operator context.
Treat grants, store helpers, mutation enforcement, adapter caches, and audits as
target behavior until implemented.

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
command visibility and completion. Snapshot staleness can affect visibility but
must not bypass daemon enforcement.
