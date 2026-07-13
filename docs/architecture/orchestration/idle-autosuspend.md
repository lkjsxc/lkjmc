# Idle autosuspend

## Purpose

Record the JSON policy shape without claiming a runtime stop or wake effect.

## Status

partial

Missing: no admitted process observation, stop, wake, queue, or player-arrival
boundary exists.

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

Network defaults, templates, and instances may store this input shape. More
specific input wins only for pure planning; it cannot launch a worker or change
a runtime.

## Fail-closed boundary

The daemon starts no reconciler, cleanup loop, autosuspend stop, wake queue, or
transfer. Every command that could cause one is `denied-unproved` before its
handler. PostgreSQL desired state, presence, and a queued record do not prove a
post-launch effect or a player outcome.

A future proposal needs an independently observed, durable completion boundary,
crash ordering, cancellation, cleanup, and bounded load evidence. It may not use
an executor, journal, actor, lease, broker, or synthetic completion to bypass
that requirement.
