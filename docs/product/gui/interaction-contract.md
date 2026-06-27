# Interaction contract

## Purpose

This contract defines deterministic inventory interaction behavior.

## Input rules

- Display text never determines behavior.
- Every plugin-rendered item carries plugin metadata for route, slot, action,
  payload, session id, render epoch, and inert state when relevant.
- Unknown display text without plugin metadata is inert.
- Unknown, stale, mismatched, or malformed plugin metadata is a framework
  failure with a localized message.
- Empty and inert slots are cancelled and silent inside plugin top inventories.
- Primary action does not depend on click type unless an owner doc defines a
  real secondary action.
- Text-entry actions close the menu, capture only the player's next chat
  message, expire after 60 seconds, and are cleared on quit.
- Destructive operations use confirmation menus.

## Reducer classifications

Pure reducers classify top-menu clicks, bottom-inventory token clicks, empty
slots, inert slots, disabled actions, stale metadata, mismatched sessions,
mismatched routes, denied actions, loading states, navigation, and real actions.

## Adapter rules

The Paper/Folia adapter owns platform effects only: inventory creation,
persistent metadata, event cancellation, scheduler crossing, daemon requests,
player messages, transfers, and token repair. Database, daemon, filesystem,
network, download, and process work must happen away from scheduler threads.
Completion callbacks that mutate inventory or player state re-enter the correct
player scheduler.

## Refresh rules

Menus refresh after successful state-changing actions. Manual refresh is allowed
for dynamic data surfaces. Background reopen loops are forbidden.
