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
known, and autosuspend reason when present. Transfer is enabled only when a Velocity path exists and the target is ready.
Admin wake transfer uses the daemon queue for suspended targets. Stopped,
starting, full, hidden, or denied targets render exact disabled reasons.

## Operator actions

Start wakes suspended instances. Stop is deliberate and uses confirmation in
menus. Restart is destructive enough to require confirmation. The proxy is never
autosuspended and should not expose stop controls to ordinary players.

## Wake-and-join target

Wake-and-join uses the daemon-owned `wake_join_queue`. A player request records
intent, starts the backend, refreshes Velocity registration, and transfers only
after the daemon start path reports success. Queue rows are marked ready or
failed; localized expiry/cancellation cleanup is still future work.

## Current boundary

The daemon queue and Velocity admin wake-send path exist. User-facing menu
suspended transfer controls remain disabled until localized expiry and
cancellation cleanup are implemented.
