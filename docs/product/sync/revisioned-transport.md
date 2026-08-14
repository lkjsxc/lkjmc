# Revisioned sync transport

## Purpose

This document owns read-only daemon-to-Java synchronization. It does not own
player save, load, application, transfer, arrival, or mutation.

## Status

implemented

## Domain contract

The closed domains are `permissions`, `claims`, `profiles`, `presence`,
`routing`, and `settings`. The removed `menus` domain is rejected. A subscription key is an opaque non-empty UTF-8 value
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

PostgreSQL owns `sync_domain_revisions`, active and archive feed tables, and
`sync_retention_policy`. Every payload query has this trigger dependency matrix:

| Payload | Owning write dependencies | Revision key |
| --- | --- | --- |
| permissions | `admin_grants`, `admin_roles` | affected principal |
| claims | `player_claims`, `claim_chunks`, `claim_trusts` | affected instance |
| profiles | typed profile snapshots | player and scope |
| presence | `instance_presence` | affected instance |
| routing | `instances`, `instance_observations`, `instance_ports`, `instance_presence` | `network` |
| settings | `player_settings` | player UUID |

A trigger touches every dependent domain/key and appends its immutable feed row
in the owning writer transaction. One transaction de-duplicates repeated touches
per key. It cannot commit a payload input change while losing any dependent
revision; rollback publishes neither. Feed revisions are global and monotonic.

Active rows are archived after 30 days and archives are deleted after 365 days.
Each bounded batch locks selected rows; a row already present in the archive is
still removed safely from the active feed. Exactly one daemon-owned maintenance
worker performs bounded retention batches off the async reactor. Each run obtains and releases a pooled PostgreSQL
connection; no connection is held during periodic sleep. Shutdown cancels and
joins the worker. Status diagnostics expose singleton count, running state,
completed runs, archived/deleted rows, and last success/error without secrets.

Polling is selected over held long-polls. A poll accepts cursor and limit
(`1..=128`), runs under daemon request admission and database statement deadline,
and returns changes, current cursor, active floor, and credential revision.
Cancellation drops the worker future and PostgreSQL statement timeout bounds
abandoned work. A cursor below the floor must clear affected cache state and
perform bounded full snapshots.

## Java coordinator

`jvm-common` can own one coordinator per plugin. Subscriptions are keyed
single-flight by domain/key. The current Paper lifecycle has no subscriptions,
and production instance credentials have heartbeat scope only; these transport
paths are internal test coverage rather than a supported player surface. The coordinator uses Java 21 `HttpClient.sendAsync`, never waits
on a Minecraft scheduler thread, and bounds subscriptions, in-flight requests,
cache entries, total bytes, age, and response bytes. Before cache mutation, a
pure domain validator requires exact JSON kinds and fields, valid
UUID/identifier forms, and documented numeric/string bounds. Revision and
cursor values must be JSON numbers whose mathematical values are integral,
nonnegative, and within the target Java integer range; positive revision fields
also reject zero. String or Boolean coercion is forbidden for every envelope
and payload field. Unknown, missing, or malformed fields reject the whole
response. A malformed newer snapshot advances neither cache, required revision,
nor feed cursor.

Snapshot and feed attempts each have single-flight failure state. Failures use
bounded exponential backoff with deterministic per-key jitter; success alone
resets that path. The retry clock is injectable for deterministic tests, and an
outage or reconnect storm remains within the configured request budget.

A credential generation change cancels in-flight requests and clears cache.
A daemon credential-revision mismatch does the same before reconnect. The
coordinator exposes an opaque durable cursor checkpoint for caller-owned reload.
Scheduler-facing disable only stops admission, cancels scheduled and in-flight
HTTP work, rejects queued results, and starts both coordinator and owned HTTP
executor shutdown. An off-scheduler await bounded to two seconds joins the
coordinator, HTTP client, and owned workers. The Compose JVM harness repeatedly
closes saturated and unavailable transports within that unchanged bound and
requires zero owned HTTP or coordinator threads afterward. No player or domain
creates a duplicate poller.

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

`scripts/check-sync-adoption.py` owns eight fail-closed probes and a standalone
Java 21 `HttpClient` harness against a real daemon route and PostgreSQL. Pure JVM
tests cover domain payload validation, cache decisions, deterministic backoff,
and maintenance lifecycle decisions. PostgreSQL tests falsify every matrix edge,
including presence-to-routing coherence. Missing database, daemon, or Java
prerequisites fail named probes; mutation tests reject an unrevisioned snapshot,
malformed cache advancement, duplicate poller/maintenance worker, missing
retention caller, and unbounded retry/cache behavior.
