# Admin model

## Purpose

This document defines the target admin roles and user-visible behavior.

## Roles

- `owner`: all admin permissions, grants, revokes, and destructive operations.
- `operator`: status, server lifecycle, reload, announcements, and reports.
- `moderator`: reports, warnings, notes, bans, mutes, and claim inspection.
- `support`: status, doctor, report viewing, and player inspection.
- `builder`: warp and claim support plus non-dangerous server information.
- `player`: default user capabilities.

## Grants

Grants identify a principal kind, principal id, role, scope, expiry, reason, and
granting actor. Revokes keep history instead of deleting rows. The first owner
must be created through a local CLI path that records audit and never prints
secrets.

## Visibility

Minecraft command visibility and completion must use one resolver that combines
platform permissions, `op`, and cached daemon grants. A stale or missing grant
snapshot may hide privileged rows, but daemon authorization remains final for
mutations.

## Audit

Grant, revoke, denied privileged action, and dangerous admin command events must
record actor, subject, action, target, result, reason, correlation id when
present, and redacted metadata.
