# Claim protection

## Purpose

This document defines non-blocking protection behavior for claims.

## Implemented rules

- Protection checks do not call the daemon from event handlers.
- Paper/Folia keeps an immutable in-memory claim snapshot refreshed
  asynchronously on a schedule and after successful mutations.
- A Java common pure protection policy models break, place, and interact
  decisions from the current snapshot.
- Block break and block place events consult the current snapshot through that
  policy; left-click and place-shaped right-click interactions are left to those
  mutation events.
- Other block interaction events consult the current snapshot through that
  policy.
- Owners, trusted players, and `lkjmc.admin.claim` operators are allowed in
  known claimed chunks.
- Strangers are denied in known claimed chunks.
- When the daemon is unavailable, known claimed chunks remain protected from the
  last snapshot.
- Unknown chunks are allowed during daemon outage rather than locking the whole
  server.
- Denial feedback is localized and rate-limited.

## Source owners

- Pure policy and snapshot cache: `platforms/jvm/common/src/main/java/com/lkjmc/common/claim`.
- Refresh adapter: `ClaimSnapshotService.java`.
- Listener: `ClaimProtectionListener.java`.
