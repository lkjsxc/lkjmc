# Server lifecycle

## Purpose

This product contract defines visible server lifecycle states and actions.

## States

- `stopped`: operator or product intent keeps the instance stopped.
- `starting`: start requested and health is not ready yet.
- `running`: process is healthy or becoming healthy.
- `suspended`: autosuspend stopped an empty eligible backend.
- `stopping`: stop requested and the process is draining.
- `failed`: last runtime action failed.

## User-facing rules

Server menus render desired state, observed state, readiness, player count when
known, connect host/port, proxy registration intent, proxy registered state when
known, joinable flag, and autosuspend reason. Transfer is enabled only when a
Velocity path exists and the target is ready and registered. Suspended public
targets expose wake-and-join when permission, queue, expiry, cancellation, and
transfer contracts are implemented. Stopped, starting, full, hidden, unknown
registration, or denied targets render exact disabled reasons.

## Operator actions

Start wakes suspended instances. Stop is deliberate and uses confirmation in
menus. Restart is destructive enough to require confirmation. Server create menus
generate ids from the selected template and carry kind/template/id in route
params. The proxy is never autosuspended and should not expose stop controls to
ordinary players.

## Wake-and-join contract

Wake-and-join uses the daemon-owned `wake_join_queue`. A player request records
actor, target instance, expiry, correlation id, and state. Duplicate live
requests for the same player and target return the existing row. Cancellation is
idempotent and never stops a server needed by other players.

## States

Rows move through `queued`, `starting`, `ready`, `transferred`, `expired`,
`cancelled`, `failed`, and `denied`. Cleanup is durable and safe on daemon
restart. Velocity consumes ready rows only after rechecking registration and
readiness, then marks transfer attempts so racing clicks cannot double-send.
