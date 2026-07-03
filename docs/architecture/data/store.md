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
- player identity, lease, snapshot, restore, and session helpers
- points balances/leaderboards, daily rewards, homes, warps, parties, achievements, shop, kits,
  vote links/rewards, reports, warnings, notes, and pending teleport helpers
- announcement, command, and audit inserts
- temporary instance, adventure session, and transfer intent data helpers

## Test contract

Store integration tests run against real PostgreSQL when
`LKJMC_STORE_TEST_DATABASE_URL` is set. Docker Compose verification sets that
environment variable and resets the test database schema before migrating.

## Current boundary

The store remains synchronous and is called by daemon adapter modules through a
pooled PostgreSQL client source. Daemon config owns `database.poolSize` with a
default of `8` and valid range `1..=64`. CLI migration paths and tests may use a
single direct connection helper.

More than one write belongs in one transaction, and audit rows for those writes
must be inserted inside the same transaction. Temporary adventure helpers can run
inside caller-owned PostgreSQL transactions for purchase, party participant
queueing, startup refund, return state, and transfer intent orchestration.
