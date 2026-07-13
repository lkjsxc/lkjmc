# Transfer safety

## Purpose

This document records the withdrawal boundary for Java profile transfer.

## Status

implemented

## Current boundary

PostgreSQL records a transfer intent bound to player, active session revision,
profile revision, lease fence, target, and correlation. The data owner may mark
an unacknowledged intent failed, but cannot record save acknowledgement or
arrival. The old audit-only `player.transfer.saved` command and temporary
transfer table are removed rather than aliased.

Paper/Folia snapshot save/load, plugin messages, Velocity transfer,
cross-server teleport, and menu transfer actions remain withdrawn. Durable
`pending_save`, `save_acknowledged`, or `pending_arrival` state is not proof that
a player moved; only an independently authenticated future arrival observation
may reach `arrived`.

## Future rule

A future adapter must obtain trusted authenticated player/session attestation,
save a fenced exact revision off the scheduler, acknowledge the same correlation
and revision, then report the actual target connection result. Duplicate exact
reports are stable. Stale fences/revisions, changed correlations, skipped states,
and unauthenticated reports are denied.

## Evidence boundary

Store tests prove record and recovery semantics only. Java containment inspection
proves no transfer bridge or daemon client is packaged.
