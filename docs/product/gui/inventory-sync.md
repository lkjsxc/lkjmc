# Inventory sync

## Purpose

This document owns token and menu inventory repair semantics.


## Status

implemented

## Repair triggers

The Paper/Folia adapter repairs the token immediately and again on delayed
player-scheduler passes after join, respawn, inventory close, cancelled token
movement, pickup completion, and menu open failure.

## Repair actions

- Remove duplicate plugin menu tokens outside player hotbar slot `8`.
- Replace slot `8` with the localized token when the setting is enabled.
- Clear slot `8` when the setting is disabled and the current item is a token.
- Call inventory update only from the correct player scheduler.

## Safety rules

Repair never blocks on daemon, filesystem, or network work. Data needed for the
token setting comes from a short-lived player cache or an async daemon request
that completes back on the player scheduler.

## Failure behavior

When settings cannot be loaded, the adapter preserves the existing token state
and sends a localized failure only when the player attempted to open the menu.
Silent background repair must not spam chat.
