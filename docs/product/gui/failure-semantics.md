# Failure semantics

## Purpose

This document defines how engine menus and command adapters communicate blocked,
invalid, or failed actions.

## Status

planned

## Frame states

The kernel phase is the complete visual failure model: `Loading`, `Loaded`,
`Empty`, `Denied`, `Stale`, `Diagnostic`, and `Static`. Every phase renders
through one frame function with shared chrome. Routes do not hand-roll separate
barrier items or close inventories for failures.

## Diagnostic codes

Only these typed codes may reach menu diagnostics:

| Code | Player-safe meaning |
|---|---|
| `daemon.not_configured` | Daemon access is not configured. |
| `daemon.config_invalid` | Daemon client config is invalid. |
| `daemon.token_missing` | Token source is absent. |
| `daemon.token_unreadable` | Token source cannot be read. |
| `daemon.auth_failed` | Daemon rejected authentication. |
| `daemon.http_connect` | Daemon connection failed. |
| `daemon.http_timeout` | Daemon request timed out. |
| `daemon.http_failed` | Daemon returned an HTTP failure. |
| `daemon.command_unknown` | Daemon command is not registered. |
| `daemon.command_failed` | Daemon command returned failure. |
| `daemon.database_not_configured` | Database config is absent. |
| `daemon.database_unavailable` | Database is unavailable. |
| `daemon.schema_mismatch` | Database schema is not current. |
| `menu.permission_denied` | Player lacks permission. |
| `menu.decode.<route>` | Binding could not decode that route. |

Each code maps to `diagnostic.<code>.title` and `diagnostic.<code>.hint`.
Player copy never includes secrets, URLs, raw JSON, stack traces, generated
secret text, or host filesystem paths beyond a sanitized file kind.

## Stale data

On `LoadData` failure the runtime consults the stale cache. A hit renders the
last good view with a warning line carrying the diagnostic short key. Navigation
entries may stay enabled. Daemon mutation entries render disabled with
`menu.stale.action-disabled`.

## Action feedback

Daemon actions declare success and failure locale keys. When a daemon response
carries a typed diagnostic code, the runtime prefers the diagnostic title and
hint over a generic action failure key. Successful state-changing actions refresh
when their action asks for refresh.

## Silent and loud cases

Silent: empty slots, inert slots, decoration, page indicator, clicks outside the
top inventory, and bottom-inventory clicks except the hotbar token.

Localized message without closing: malformed metadata, stale metadata, route or
session mismatch, disabled rows, denied actions, and text-input cancellation.

Never allowed: unintended inventory closes, fake success messages, raw error
text, or hidden mutation attempts.

## Logging

The runtime logs diagnostics at WARN with code and route id only. Repeated
identical codes within one minute collapse behind a per-code timestamp guard.
