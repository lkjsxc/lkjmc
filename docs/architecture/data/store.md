# Store

## Purpose

This document defines the implemented Rust store foundation.

## Implemented boundary

`lkjmc-store` provides synchronous PostgreSQL helpers for the current schema
foundation:

- database connection setup from a URL
- ordered migration discovery and application
- migration ledger reads
- node insert and read
- jar asset insert and read
- instance insert, read, observation upsert, and wake-and-join queue helpers
- player identity, lease, snapshot, restore, and session helpers
- points balances/leaderboards, daily rewards, homes, warps, parties, achievements, shop, kits,
  vote links/rewards, reports, warnings, notes, and pending teleport helpers
- announcement, command, audit, and outbox inserts
- temporary instance, adventure session, and transfer intent data helpers

## Test contract

Store integration tests run against real PostgreSQL when
`LKJMC_STORE_TEST_DATABASE_URL` is set. Docker Compose verification sets that
environment variable and resets the test database schema before migrating.

## Current boundary

The store remains synchronous and is called by daemon adapter modules.
Connection pooling and async query adapters remain outside the current boundary.
Temporary adventure helpers can run inside caller-owned PostgreSQL transactions
for live End Expedition purchase, party participant queueing, startup refund,
participant return state, and transfer intent orchestration.
