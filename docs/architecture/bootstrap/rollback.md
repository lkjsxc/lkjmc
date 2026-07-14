# Bootstrap rollback

## Purpose

This contract defines rollback behavior for failed bootstrap applies.


## Status

implemented

## Safe rollback

Rollback effects are planned before apply. They may stop only a process started
under the current fenced attempt and restore only an atomically replaced file
whose prior digest was observed by that attempt. Immutable assets, generated
secrets, and pre-existing processes are never deleted speculatively. When safe
rollback cannot be proved, the failed partial observation remains durable and
the next fresh inspection emits repair changes.

## Non-rollback state

Immutable assets, generated secrets, and already-managed instance records are not
deleted automatically after a later effect fails. They are durable facts for the
next idempotent run unless the operator asks for removal through a real command.

## Step recording

Each admitted apply creates a `bootstrap_runs` record and records every effect
in `bootstrap_steps` with order, target, result, and diagnostic. A later apply
marks an unfinished run failed before creating its own run. Failures must be
stored without secrets.

## Retry rule

A later apply must gather fresh facts and converge from the partial state rather
than replaying stale assumptions from a failed run. Correlation, authored
revision, database revision, failed step, diagnostic, and observed identities
remain queryable. Diagnostics contain paths, ids, and digests, never secret
contents.
