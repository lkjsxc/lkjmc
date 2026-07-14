# Revisioned sync transport

## Purpose

This document owns read-only daemon-to-Java synchronization. It does not own
player save, load, application, transfer, arrival, or mutation.

## Status

implemented

## Domain contract

The closed domains are `permissions`, `claims`, `menus`, `profiles`, `presence`,
`routing`, and `settings`. A subscription key is an opaque non-empty UTF-8 value
of at most 256 bytes. Every successful read returns exactly one immutable typed
envelope:

```json
{
  "domain": "claims",
  "key": "survival",
  "revision": 42,
  "generatedAt": "2026-07-14T00:00:00.000000Z",
  "payload": {}
}
```

`revision` is positive and monotonic for a domain/key. `generatedAt` is database
transaction time, not freshness authority. Payload shapes are domain-specific;
a client rejects a domain/key mismatch or malformed payload. A missing key or
PostgreSQL failure returns `unavailable`; a cursor below the active retained
floor returns `reload-required`. Neither result supplies stale data as current.

`profiles` exposes typed durable envelopes only. Java may cache those bytes but
must not apply them to a player. Trusted A-JVM session attestation is still
required before external save, load, transfer, arrival, or acknowledgement.

## PostgreSQL architecture

PostgreSQL owns `sync_domain_revisions`, `sync_change_feed`, and
`sync_retention_policy`. Table triggers map each owned write to affected
keys and atomically increment the key revision plus append one immutable feed
row in the writer transaction. Triggers cover grants, claims, catalogs, typed
profiles, presence, topology, and settings; a write cannot commit its data while
losing its sync revision. Feed revisions are global and monotonic.

Active feed rows are retained for 30 days. Polling is selected over held
long-polls: the real Java/daemon harness shows bounded polling meets the
five-second freshness bound while occupying no database connection between
requests. A poll accepts cursor and limit (`1..=128`), runs under daemon request
admission and database statement deadline, and returns changes, current cursor,
active floor, and credential revision. Cancellation drops the worker future and
PostgreSQL statement timeout bounds abandoned work. A cursor below the floor
must clear affected cache state and perform bounded full snapshots.

## Java coordinator

`jvm-common` owns one coordinator per plugin. Subscriptions are keyed
single-flight by domain/key; Paper and Velocity own lifecycle and immutable view
adapters only. The coordinator uses Java 21 `HttpClient.sendAsync`, never waits
on a Minecraft scheduler thread, and bounds subscriptions, in-flight requests,
cache entries, total bytes, age, and response bytes. It applies only increasing
revisions, repairs loss with full reload, and uses capped exponential
backoff with deterministic per-key jitter.

A credential generation change cancels in-flight requests and clears cache.
A daemon credential-revision mismatch does the same before reconnect. The
coordinator exposes an opaque durable cursor checkpoint for caller-owned reload.
Scheduler-facing disable only stops admission, cancels work, and starts executor
shutdown; an off-scheduler bounded await proves clean termination. No player or
domain creates a duplicate poller.

## Security and failure boundary

Only a live scoped credential with `lkjmc.sync.read` may use loopback `/sync`.
Authentication checks PostgreSQL's credential revision on every request, so
rotation, revocation, expiry, and database uncertainty fail closed. Tokens are
construction inputs, are never logged, and changing a token requires explicit
coordinator credential replacement.

The transport is read-only. It registers no Minecraft command, dynamic action,
claim listener, grant authority, routing mutation, profile bridge, or transfer
bridge. Cached permissions are presentation hints and never authorization
proof.

## Proof

`scripts/check-sync-adoption.py` owns seven fail-closed probes and a standalone
Java 21 `HttpClient` harness against a real daemon route and PostgreSQL. Pure JVM
tests are limited to cache and backoff decisions. Missing database, daemon, or
Java prerequisites fail named probes; mutation tests prove an unrevisioned
snapshot, duplicate poller, and unbounded cache are rejected.
