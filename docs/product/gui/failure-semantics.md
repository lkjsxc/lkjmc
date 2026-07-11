# Failure semantics

## Purpose

This document defines how the local documentation menu communicates blocked or
invalid local behavior.

## Status

implemented

## Frame states

The local kernel phase is the complete visual failure model: `Loaded`, `Empty`,
`Diagnostic`, and `Static`. Every phase renders through one frame function with
shared chrome. Routes do not close inventories for failures.

## Diagnostic codes

| Code | Player-safe meaning |
| --- | --- |
| `menu.local_content_invalid` | Bundled documentation content is invalid. |
| `menu.local_content_missing` | Bundled documentation content is unavailable. |
| `menu.decode.<route>` | Local binding could not decode that route. |

Each fixed code maps to a localized title and hint. Player copy never includes
secrets, URLs, raw JSON, stack traces, generated secret text, or host filesystem
paths.

## Action feedback

The shipped menu has no daemon action or state-changing action. Local navigation,
search, and external-link presentation use localized failure copy when needed.
Malformed local metadata, stale session metadata, disabled rows, and cancelled
text input preserve the open inventory.

## Silent and loud cases

Silent: empty slots, inert slots, decoration, page indicator, clicks outside the
top inventory, and bottom-inventory clicks except the hotbar token.

Never allowed: unintended inventory closes, fake success messages, raw error
text, hidden mutation attempts, or a fallback daemon action.

## Logging

The runtime logs local diagnostics at WARN with code and route id only. Repeated
identical codes within one minute collapse behind a per-code timestamp guard.
