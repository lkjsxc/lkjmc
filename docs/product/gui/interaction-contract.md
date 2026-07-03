# Interaction contract

## Purpose

This contract defines deterministic inventory interaction behavior.


## Status

implemented

## Input rules

- Display text never determines behavior.
- Every plugin-rendered item carries plugin metadata for route, slot, action,
  deterministic payload fields, session id, render epoch, and inert state when
  relevant.
- Unknown display text without plugin metadata is inert.
- Unknown, stale, mismatched, or malformed plugin metadata is a framework
  failure with a localized message.
- Empty and inert slots are cancelled and silent inside plugin top inventories.
- Primary action does not depend on click type unless an owner doc defines a
  real secondary action.
- Text-entry actions are allowed only for free-form names, reasons, messages, or
  announcements when no safe picker exists; they keep the session alive, prompt
  for the player's next chat message, state cancel behavior, expire after 60
  seconds, and are cleared on quit.
- Destructive operations use confirmation menus carrying the exact selected
  object id, preconditions, and force flag when applicable. Cancel is true Back.

## Reducer classifications

Pure reducers classify top-menu clicks, bottom-inventory token clicks, empty
slots, inert slots, disabled actions, stale metadata, mismatched sessions,
mismatched routes, denied actions, loading states, navigation, and real actions.
Visible `menu.back` metadata always resolves to a Back effect, not a parent-route
OpenRoute effect. Payload decoding preserves all sorted key/value fields; it must
not drop multi-field context such as `id`, `force`, and `reason`.

## Adapter rules

The Paper/Folia adapter owns platform effects only: inventory creation,
persistent metadata, event cancellation, scheduler crossing, daemon requests,
player messages, transfers, and token repair. Database, daemon, filesystem,
network, download, and process work must happen away from scheduler threads.
Completion callbacks that mutate inventory or player state re-enter the correct
player scheduler.

## Close rules

No menu-owned action may close the inventory except the explicit close button.
Back, refresh, route changes, disabled rows, command parity actions, daemon
actions, dynamic replacements, and text prompts must preserve the session and
route stack. Manual player closes still clear the session.

## Refresh rules

Menus refresh after successful state-changing actions. Manual refresh is allowed
for dynamic data surfaces and preserves the route stack. Loading and unavailable
replacement preserve the route stack as well. Background reopen loops are
forbidden.
