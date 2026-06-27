# Presence

## Purpose

This document defines durable player presence separate from process health.

## Table

`instance_presence` stores one row per instance with the latest heartbeat time,
player count when known, max players, ready flag, empty timing, suspend and wake
timestamps, suspend reason, metadata, and update time.

## Heartbeat body

```json
{
  "id": "hub",
  "playerCount": 0,
  "maxPlayers": 20,
  "ready": true,
  "implementation": "folia"
}
```

If a heartbeat omits player count, presence is unknown rather than empty. The
daemon still records process health from the heartbeat.

## Store helpers

`lkjmc-store` owns typed helpers to upsert heartbeat presence, read by instance,
list presence for reconciliation, set or clear empty state, mark autosuspended,
and clear autosuspend on explicit start.

## Consumers

Autosuspend uses presence and active sessions together. Status and instance list
surfaces expose presence summaries so operators can explain why an instance is
warm, empty, suspended, or skipped.
