# Store

## Purpose

This document defines the implemented Rust store foundation.


## Status

implemented

## Implemented boundary

`lkjmc-store` provides synchronous PostgreSQL helpers for the current schema
foundation:

- PostgreSQL pool construction from a URL with configurable maximum size
- ordered migration discovery and application
- migration ledger reads
- node insert and read
- jar asset insert and read
- instance insert, read, observation upsert, and wake-and-join queue helpers
- transactional player identity, session, fenced lease, typed snapshot, and restore helpers
- points balances/leaderboards, daily rewards, homes, warps, parties, achievements, shop, kits,
  vote links/rewards, reports, warnings, notes, and pending teleport helpers
- announcement, command, and audit inserts
- temporary instance and fenced adventure lifecycle helpers
- transfer, item-delivery, and runtime intent/observation workflow helpers
- monotonic workflow change-feed archive and retention helpers

## Test contract

Store integration tests run against real PostgreSQL when
`LKJMC_STORE_TEST_DATABASE_URL` is set. Docker Compose verification sets that
environment variable and resets the test database schema before migrating.

## Current boundary

The store remains synchronous and is called by daemon adapter modules through a
pooled PostgreSQL client source. Daemon config owns `database.poolSize` with a
default of `8` and valid range `1..=64`. CLI migration paths and tests may use a
single direct connection helper.

More than one write belongs in one transaction, and its audit/change rows are
inserted inside that transaction. `config/data-workflows.json` names the owner
for every classified multi-write and effect boundary. Profile writes compare
session revision, lease fence, snapshot revision, and correlation while holding
row locks. An exact replay returns the original result; a changed replay or
stale value is denied.

Workflow transition decisions are pure. Store adapters lock the aggregate,
apply one legal compare-and-swap transition, and append its change row. Data-only
callers may create durable external-effect intent, but cannot record receipt,
arrival, cleanup observation, or runtime success without the future trusted
acknowledgement type. Missing acknowledgement remains pending or becomes failed.
