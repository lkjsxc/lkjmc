# Admin model

## Purpose

This document defines durable admin roles and their attested operator boundary.

## Status

implemented

## Roles

- `owner`: all admin permissions, grants, revokes, and destructive operations.
- `operator`: status, server lifecycle, reload, announcements, and reports.
- `moderator`: reports, warnings, notes, bans, mutes, and claim inspection.
- `support`: status, doctor, report viewing, and player inspection.
- `builder`: warp and claim support plus non-dangerous server information.
- `player`: default user capabilities.

## Grants

Grants identify a principal kind, principal id, role, scope, expiry, reason, and
granting actor. Revokes keep history instead of deleting rows. The CLI and web
surface can grant, revoke, inspect, and tail admin audit through daemon commands.
The first owner is a local CLI grant operation and never prints secrets.

## Visibility

Java `/lkjmc` visibility, completion, and admin menu rows are withdrawn pending
trusted identity/session attestation. Paper and Velocity do not cache grants.
An attested CLI or web subject receives daemon-final authorization; a cached
value, platform permission, `op`, or actor-shaped request is not proof.

## Audit

Grant, revoke, denied privileged action, and dangerous admin command events must
record actor, subject, action, target, result, reason, correlation id when
present, and redacted metadata.
