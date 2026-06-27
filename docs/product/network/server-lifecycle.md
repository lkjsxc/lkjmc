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

## Future wake path

Wake-and-join requires a real queue, timeout, transfer retry, and player-facing
failure path. Until then, transfers to suspended servers fail with an exact
reason rather than silently starting a server.
