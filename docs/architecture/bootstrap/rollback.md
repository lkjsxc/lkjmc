# Bootstrap rollback

## Purpose

This contract defines rollback behavior for failed bootstrap applies.


## Status

implemented

## Safe rollback

Rollback effects must be explicit and safe. They may stop processes started by
the failed run, remove files written under a run-owned temporary path, and mark
database run steps failed.

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
than replaying stale assumptions from a failed run.
