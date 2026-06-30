# Web control surface

## Purpose

This document preserves web UI guardrails for a future proposal. Web UI is not a
current product target under
[control surface scope](../decisions/control-surface-scope.md).

## API contract

If a future decision adds a web surface, it must call the same daemon command API
used by the CLI and plugins. It may add presentation routes, but mutating actions
go through daemon handlers, use stable JSON request and response shapes, and
write audit events.

## Security contract

The web listener binds privately by default, authenticates every request, avoids
printing or exposing daemon tokens, and denies public control unless an operator
explicitly configures a safe front door.

## Verification target

A future web slice should test daemon API calls, token failures, private bind
configuration, static UI behavior if present, and audit coverage for mutating
actions.

## Current status

No web UI code is implemented or registered, and none should be added until the
scope decision is changed.
