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

- process absent.
- process starting.
- process healthy.
- process unhealthy.
- process exited.
- process unknown.

## Reconciliation

The daemon reads desired state, process observations, presence, active player
sessions, and policy. Pure planning decides whether to start, stop, mark empty,
clear empty, mark suspended, or skip with a reason. The adapter writes durable
state before process effects so the next tick does not fight the plan.

## Manual wake

`instance.start` clears autosuspend fields, writes desired state `running`, and
starts the runtime. Explicit `instance.stop` writes deliberate `stopped`; it is
not treated as autosuspend.
