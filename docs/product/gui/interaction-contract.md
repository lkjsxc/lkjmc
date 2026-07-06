# Interaction contract

## Purpose

This contract defines deterministic inventory interaction behavior for the
implemented menu engine.

## Status

implemented

## Input rules

- Display text never determines behavior.
- Every non-inert engine item carries metadata for route, params, slot, action
  key, payload, session id, and render epoch.
- Unknown display text without plugin metadata is inert.
- Unknown, stale, mismatched, or malformed plugin metadata is a framework
  failure with a localized message.
- Empty and inert slots are cancelled and silent inside engine top inventories.
- Primary action does not depend on click type unless an owner doc defines a
  real secondary action.
- Text-entry actions are allowed only for free-form names, reasons, messages,
  or announcements when no safe picker exists. They keep the session alive,
  store the command prefix with the prompt, prompt for the player's next chat
  message, state cancel behavior, expire after 60 seconds, and clear on quit.
- Destructive operations use confirmation routes carrying the exact selected
  object id, preconditions, and force flag when applicable. Cancel is true Back.

## Metadata validation

The kernel validates clicks against the current model. Malformed or unknown
metadata maps to `UNKNOWN_METADATA`; session mismatch to `STALE_SESSION`; epoch
mismatch to `STALE_EPOCH`; route mismatch to `ROUTE_MISMATCH`. Inert slots and
empty slots are silent no-effect decisions.

## Action grammar

Valid clicks resolve to document or binding actions: open a route, Back, Close,
Refresh, run a player command, send a daemon command, transfer a player, send a
message, or prompt for text. A decision that does no work returns an empty
effect list.

## Adapter rules

The Paper runtime owns platform effects only: inventory creation, persistent
metadata, event cancellation, scheduler crossing, daemon requests, player
messages, transfers, text input, stale cache, and token repair. Database,
daemon, filesystem, network, download, and process work must happen away from
scheduler threads. Completion callbacks that mutate inventory or player state
re-enter the correct player scheduler.

## Render and close rules

When the same session is open and the frame size is unchanged, refresh and data
replacement mutate item stacks in place instead of reopening the inventory. A
full open happens only on route change, size change, or when no engine inventory
is open.

No menu-owned action may close the inventory except the explicit Close slot.
Back, refresh, route changes, disabled rows, player commands, daemon actions,
dynamic replacements, and text prompts preserve the session and route stack.
Manual player closes still clear the session.
