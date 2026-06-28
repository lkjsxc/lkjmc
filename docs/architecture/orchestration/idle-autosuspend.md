# Idle autosuspend

## Purpose

This document defines how eligible empty backend instances stop to free memory.

## Policy JSON

```json
{
  "autosuspend": {
    "enabled": true,
    "idleGraceSeconds": 300,
    "minimumUptimeSeconds": 120,
    "heartbeatStaleSeconds": 90,
    "emptyHeartbeatCount": 2,
    "stopWhenEmpty": true,
    "deleteWhenExpired": false,
    "keepWarm": false
  }
}
```

Policy may be supplied at network default, template, and instance levels. The
more specific level wins.

## Defaults

Velocity is never autosuspended and is always warm. The default Folia hub stays
warm by policy; non-entry Paper, Folia, and Purpur backends autosuspend after
300 empty seconds. Suspended joins must use the daemon wake-and-join queue so a
player is recorded before the runtime starts. Temporary adventure instances use
a shorter grace and may be deleted or archived by their owner contract.

## Planner rules

- Never autosuspend a proxy or `keepWarm` instance.
- Never autosuspend when player count is unknown or heartbeat is stale.
- Never autosuspend while active sessions are greater than zero.
- Respect minimum uptime before stopping.
- Require consecutive empty evidence and idle grace after `empty_since`.
- Write desired state `suspended` before stopping the runtime.
- Queue suspended join requests before waking a backend.
- Audit every autosuspend, queued wake, and manual wake.

## Reconciler boundary

The reconciler refreshes local observations, loads instances and presence,
computes plans, writes state, executes process effects, and records audit data.
It must not hold the runtime mutex while making PostgreSQL calls.
