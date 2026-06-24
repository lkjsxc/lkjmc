# CLI

## Purpose

This document defines the target SSH-friendly operator surface.

## Rules

- Human output is compact and stable.
- `--json` emits machine-readable JSON without decoration.
- Failures return non-zero exit codes.
- Destructive commands require `--yes` outside interactive terminals.
- Normal commands use the daemon API instead of writing PostgreSQL directly.

## Current status

The CLI is not implemented yet.
