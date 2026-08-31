# Autosuspend

## Purpose

This operator contract explains how idle backend autosuspend behaves.


## Status

implemented

## Scope

The configured Velocity kind is never autosuspended. Any backend may be marked
keep-warm by policy.
Non-entry Paper, Folia, and Purpur backends may stop after they are empty for
the configured grace period. Temporary adventure instances use their owner
cleanup policy.

## Operator expectations

An autosuspended instance has desired state `suspended`, absent or stopped
process observation, presence fields showing the empty period, and an audit
event. Manual `lkjmc instance start <id>` wakes it and clears autosuspend fields.
Player joins to suspended backends go through `instance.wake.request`, which
records a queue row, wakes the backend, and returns the target only after the
start adapter reports success. Manual `lkjmc instance stop <id>` remains
deliberate stopped state.

## Safe skips

Autosuspend skips when player count is unknown, heartbeat is stale, active
sessions exist, minimum uptime has not elapsed, the instance is keep-warm, or
the instance kind is Velocity.

## Verification

Unit tests prove planner rules. Integration tests prove heartbeat persistence,
state writes before stop, and that the reconciler does not restart a suspended
instance. Live tests must report a real backend stopping after grace and staying
stopped until manually started.
