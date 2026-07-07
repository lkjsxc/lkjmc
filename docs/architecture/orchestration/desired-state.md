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

The daemon reads desired state, runtime observations, presence, active player
sessions, and policy. Pure planning decides whether to start, stop, mark empty,
clear empty, mark suspended, or skip with a reason. Lifecycle commands record
successful desired transitions only after the runtime effect and observation
write succeed; failures leave an honest failure observation for retry.

## Manual wake

`instance.start` clears autosuspend fields, writes desired state `running`, and
starts the runtime. Explicit `instance.stop` writes deliberate `stopped`; it is
not treated as autosuspend.
