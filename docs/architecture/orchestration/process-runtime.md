# Process runtime

## Purpose

This document defines the local process runtime target contract.

## Rules

- Instances live under `/var/lib/lkjmc/instances/{id}`.
- Source instance config lives under `/etc/lkjmc/instances/{id}.json`.
- Logs live under `/var/log/lkjmc/instances/{id}`.
- Server jars come from `/opt/lkjmc/jars`.
- Process groups are used for reliable stop and kill.
- Graceful stop uses stdin or RCON when available, then signals, then kill.
- Deletion refuses active player sessions unless forced and audited.

## Current status

No process runtime is implemented yet.
