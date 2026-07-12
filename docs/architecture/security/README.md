# Security architecture

## Purpose

This area owns threat boundaries, secret handling, permissions, and safe file
access.


## Status

implemented

## Table of contents

- [Permissions](permissions.md)
- [Secrets](secrets.md)

## Authentication boundary

Minecraft fields, plugin permissions, and command bodies are untrusted. The
command registry is the complete surface policy: a stored, unexpired `cli` or
`web` credential may invoke only a command that declares that surface and a
scope matching its authorization class. The configured web bootstrap secret
creates a bounded web session; it is not TCP command authority.

The Unix socket admits only kernel peers whose UID owns the socket or whose GID
owns its `0660` group. Such peers are authenticated as the local CLI surface.
Paper, Velocity, and Discord credentials and adapters remain unavailable;
there is no compatibility credential for a withdrawn adapter.

## Cache, denial, and audit

Credential cache reads first verify the PostgreSQL credential revision. A
revision change drops cached entries; database, revision, cache, or worker
uncertainty denies rather than using a stale entry. Database work runs on a
blocking worker, never the async transport reactor. Denials audit only a
surface, safe reason, and redacted target. Secrets and submitted credentials
never enter a response, log, or audit row.

## Verification

`cargo test -p lkjmc-daemon --bin lkjmc-daemon` covers forged request fields,
peer-policy rejection, cache revision eviction, web session/CSRF expiry, login
rate limiting, and secret-provider denial. `scripts/check-security-probes.py`
checks the closed registry policy and transport containment invariants.
