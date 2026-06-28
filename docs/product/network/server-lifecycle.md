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
known, and autosuspend reason when present. Transfer is enabled only when a
Velocity path exists and the target is ready. Stopped, suspended, starting,
full, hidden, or denied targets render exact disabled reasons.

## Operator actions

Start wakes suspended instances. Stop is deliberate and uses confirmation in
menus. Restart is destructive enough to require confirmation. The proxy is never
autosuspended and should not expose stop controls to ordinary players.

## Wake-and-join target

Wake-and-join requires a daemon-owned queue. A player request enqueues intent,
starts the backend, waits for readiness, retries transfer through Velocity,
expires with a localized failure on timeout, and clears state on success,
disconnect, stop, or cancellation.

## Current boundary

Until the queue exists, transfers to suspended servers fail with an exact reason
rather than silently starting a server. Menus must keep suspended transfer
controls disabled and show the queue feature as unavailable.
