# Failure semantics

## Purpose

This document defines how menus and command adapters communicate blocked,
invalid, or failed actions.

## Status

partial

Missing: one shared JVM daemon diagnostic component and stale dynamic menu data
on every daemon-backed route.

## Silent cases

Decoration, info panels, selected summaries, page indicators, empty slots, and
unknown display text without plugin metadata are silent and inert.

## Typed diagnostics

Known daemon-backed failures map to exact safe classes: daemon HTTP not
configured, runtime config invalid, token source missing, token file unreadable,
auth rejected, HTTP connect failure, HTTP timeout, command unknown, command
failed, database not configured, database unavailable, schema mismatch,
permission denied, malformed response, and stale route context.

Player copy names the class and safe next step. It never includes secrets, token
values, database URLs, raw JSON, stack traces, generated secret text, or host
filesystem paths beyond a sanitized file kind.

## Dynamic route fallback

A route with recent successful data may render stale data with a warning when a
transient daemon failure occurs. If no stale data exists, it renders a typed
unavailable surface. A valid empty list must not be confused with daemon
failure.

## Action integrity

If a state-changing daemon action fails after validation, the menu refreshes or
reopens with the prior truthful state. Purchase-like flows state whether no
charge occurred or a refund was recorded. Enabled rows must not register fake
success.

## Disabled item copy

Disabled lore states the exact reason, next possible player step, and whether
the state is temporary, denied, unavailable, locked, loading, stale, or a typed
dependency diagnostic.

## Verification

Unit tests cover diagnostic classification and redaction. Menu gateway tests
separate empty, stale, and failed data. Command adapter tests assert exact error
mapping for known daemon codes.
