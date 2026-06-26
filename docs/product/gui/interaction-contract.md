# Interaction contract

## Purpose

This contract defines metadata-driven inventory menu behavior.

## Input rules

- Display text never determines behavior.
- Every interactive item carries action metadata: menu id, slot, action key, and
  optional payload.
- Inert items carry an inert marker.
- Unknown metadata is a framework error with a localized failure message.
- Unknown display text without plugin metadata is inert.
- Primary actions do not depend on left-click versus right-click unless an owner
  doc defines that difference.
- Destructive operations use dedicated confirmation menus.

## Reducer outcomes

A pure reducer classifies each click as one of:

- action item;
- navigation item;
- inert item;
- empty slot;
- outside menu;
- stale or unknown metadata.

Top-inventory clicks in plugin menus are cancelled before reduction. Empty and
inert clicks are consumed silently. Disabled clicks send the documented reason.

## Adapter rules

The Paper adapter owns only platform effects:

- custom inventory holder with menu id and session id;
- player session registry for current menu state;
- persistent data on rendered plugin items;
- cancelled click and drag events for plugin menu top slots;
- close cleanup for temporary session state;
- bottom-inventory clicks allowed only when they do not involve protected menu
  tokens or plugin menu state;
- scheduler-safe effect execution.

Daemon, filesystem, network, and process effects must run off scheduler threads
and return to the player scheduler before inventory mutation or messages.
