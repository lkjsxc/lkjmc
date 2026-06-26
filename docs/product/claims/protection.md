# Claim protection

## Purpose

This document defines the target non-blocking protection behavior for claims.

## Target rules

- Protection checks must not call the daemon on the event thread.
- Paper/Folia keeps an immutable in-memory claim snapshot refreshed
  asynchronously on a schedule and after successful mutations.
- Block break, block place, and basic block/container interactions consult the
  current snapshot.
- Owners, trusted players, and operators are allowed in known claimed chunks.
- Strangers are denied in known claimed chunks.
- When the daemon is unavailable, known claimed chunks remain protected from the
  last snapshot.
- Unknown chunks are allowed during daemon outage rather than locking the whole
  server.
- Denial feedback is localized and rate-limited enough to avoid chat spam.

## Current status

No claim cache or protection listener exists yet.
