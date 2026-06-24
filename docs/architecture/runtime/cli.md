# CLI

## Purpose

This document defines the implemented and target SSH-friendly operator surface.

## Implemented commands

- `lkjmc doctor`
- `lkjmc status`
- `lkjmc config check --path PATH`
- `lkjmc db migrate`
- `lkjmc db status`
- `lkjmc audit tail --lines N`

`doctor`, `status`, and `audit tail` use the daemon Unix socket. Database
migration and status use `LKJMC_DATABASE_URL` directly. `--json` emits compact
machine-readable JSON for implemented commands.

## Current boundaries

Instance, jar, player restore, and log commands are not implemented or
registered yet.
