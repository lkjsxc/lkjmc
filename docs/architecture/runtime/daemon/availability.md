# Daemon availability

## Purpose

This architecture contract defines shared JVM daemon connectivity diagnostics
and stale-data behavior.

## Status

implemented

Implemented: a shared Paper/Velocity diagnostic component, token-file readability
state, and bounded stale data caches on all dynamic menu routes.

## Classification

Adapters classify missing daemon HTTP config, invalid runtime config, token
source missing, token file unreadable, auth rejected, HTTP connect failure, HTTP
timeout, command unknown, command failed, database not configured, database
unavailable, schema mismatch, permission denied, malformed body, and stale route
context separately.

## Shared component

Paper and Velocity use a common daemon access component that validates config,
tracks token source readability, records last success and failure class, supports
token-file rotation, supplies a daemon client only when usable, and exposes safe
operator hints.

The component never exposes token values, generated secrets, database URLs, raw
stack traces, or unsanitized daemon bodies to players.

## Stale data

Dynamic routes cache the last successful loaded payload per player and route for
a bounded time. On transient failures they keep the menu open and render stale
data with a warning. Without stale data they render a typed unavailable surface.
Valid empty lists remain distinct from failures.

## Command mapping

Command adapters map known daemon codes to command-class messages. Broad copy
such as `Daemon is unavailable` or `daemon command failed` is reserved only for
unknown cases after classification fails.

## Verification

Unit tests cover classification, redaction, token-file states, auth failure,
HTTP failure, command failure, database failure, schema mismatch, stale-data
selection, and empty-list separation.
