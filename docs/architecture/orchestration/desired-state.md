# Desired state

## Purpose

This document defines managed instance intent and reconciliation semantics.


## Status

implemented

## Desired states

- `stopped`: deliberate operator or product stop.
- `starting`: start requested and not yet healthy.
- `running`: desired process should be running.
- `suspended`: autosuspend stopped an otherwise runnable backend.
- `stopping`: stop requested and not yet absent.
- `restarting`: stop then start requested.
- `deleting`: delete workflow is in progress.
- `failed`: runtime action failed and needs operator attention.

## Observed states

The database accepts process states, Kubernetes adapter states, and the
adapter-neutral runtime vocabulary used for compatibility migrations:

- process absent, starting, healthy, unhealthy, exited, or unknown.
- Kubernetes absent, starting, ready, unhealthy, exited, or unknown.
- runtime absent, starting, ready, unhealthy, exited, or unknown.

## Reconciliation

The daemon reads desired state, current fenced runtime operation, latest
observation, presence, sessions, policy, and adapter capabilities. A pure
planner returns start, stop, observe, or no-op. Every pass appends an intent and
outcome, including a durable no-op outcome. A satisfied desired state does not
repeat an effect; a pending operation after restart is observed before any
repeat effect.

Intent allocation atomically increments the per-instance fence and stores an
operation and correlation id. Ownership is checked in a fresh transaction before
the external effect and again before committing observation. Stale observations
are retained in history but cannot replace current state. Deadline expiry leaves
pending/unknown or failed state and never reports success.

## Read-only topology

The `routing/network` sync snapshot contains bounded desired/observed instance
and presence summaries. It is presentation data only: a Java cache cannot
register a server, choose a transfer, acknowledge an effect, or alter desired
state.

## Manual wake

`instance.start` clears autosuspend fields and commits running intent before the
runtime effect. `instance.stop` commits deliberate stopped intent and is not
autosuspend. Start, stop, restart, and reconcile all use this one fenced path;
there is no direct alternate lifecycle route.
