# Interaction contract

## Purpose

This contract defines local documentation inventory interactions.

## Status

implemented

## Input rules

Only local document, paging, Documentation, and Close items are actionable.
Clicks in the local docs inventory are cancelled. Unknown items and empty slots
are inert. The hotbar token is locked to slot `8` and opens the same local list.

## Safety

A local action can open a bundled document, move a page, return to the list, or
close the inventory. It cannot run a player command, send a daemon command,
transfer a player, mutate state, prompt for text, or read a credential.

## Render and close rules

A document view opens a bounded 54-slot inventory. Only the explicit Close item
closes it; all local navigation preserves the player inventory and performs no
external work.
