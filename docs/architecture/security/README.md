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

## Cache, revocation, denial, and audit

A cache entry contains only an unexpired credential record. It is bounded to
128 entries and deterministically evicts the lowest token-hash key when full;
expired entries are removed before every lookup or insert. Before every cache
lookup, authentication starts a PostgreSQL transaction and takes a shared lock
on the singleton credential revision. Issuance, revocation, or an actual
credential-policy change takes the conflicting revision write lock and bumps
that revision. `last_used_at` touches never bump it.

This is the authorization ordering: an authentication holding the shared lock
may succeed and linearizes before a waiting revocation; a revocation holding or
committing the write lock makes later authentication see its new revision and
deny the old credential. A cache hit is therefore not claimed to revoke an
already-authorized request mid-flight. Revision, transaction, cache, database,
or worker uncertainty denies; no stale-cache fallback exists. Database work
runs on a blocking worker, never the async transport reactor.

Creation and successful revocation write redacted audit events with command
actor, operation, credential kind, and credential id only. Values, hashes,
scopes, principals, output paths, bootstrap secrets, and submitted credentials
never enter an audit row, response, or log. Denials audit only a surface, safe
reason, and redacted target.

## Verification

`cargo test -p lkjmc-daemon --bin lkjmc-daemon` covers forged request fields,
peer-policy rejection, cache revision eviction, web session/CSRF expiry, login
rate limiting, and secret-provider denial. `scripts/check-security-probes.py`
checks the closed registry policy and transport containment invariants.
