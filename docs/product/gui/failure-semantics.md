# Failure semantics

## Purpose

This document defines how menus communicate blocked or invalid actions.

## Silent cases

Decoration, info panels, selected summaries, page indicators, empty slots, and
unknown display text without plugin metadata are silent and inert.

## Localized failure cases

- Malformed metadata: reopen the menu.
- Session mismatch: the menu is stale and stale dynamic replacement is ignored.
- Render epoch mismatch: refresh the menu.
- Route mismatch: reopen from the current menu path without pushing history.
- Disabled action: show the documented reason key.
- Daemon not configured: state that daemon HTTP configuration is absent.
- Token missing or unreadable: state that daemon authentication cannot be read.
- HTTP failure or auth failure: state the safe daemon connectivity class.
- Daemon command unknown or failed: name the command class without stack traces.
- Database not configured or unavailable: state the database dependency class.
- Schema mismatch: state that the daemon response did not match the menu contract.
- Permission denial: state the missing permission or role.
- Loading state: explain that data is still loading.
- Adapter failure: report that the action failed and no durable state changed.
- Wake-and-join final states: queued, ready, transferred, cancelled, expired,
  failed, denied, and unavailable each have localized copy.
- Adventure purchase failure: report whether no charge occurred or a refund was
  recorded through the points ledger.
- Menu transfer failure: show a localized save-first transfer failure reason for
  unknown, unjoinable, timed-out, or denied targets.

## Disabled item copy

Disabled items render a disabled material and include the exact reason, the next
possible player step, and whether the state is temporary, denied, unavailable,
locked, loading, or a typed dependency diagnostic. Disabled items never register
fake action effects.

## State integrity

If a state-changing daemon action fails after validation, the menu refreshes or
reopens with the prior truthful state. Back fallback repairs to root rather than
leaving the current route out of sync with the stack. If a purchase-like future
action cannot commit atomically, it must not be registered as live.
