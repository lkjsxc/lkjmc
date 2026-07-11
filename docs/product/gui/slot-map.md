# Slot map

## Purpose

This document defines the local documentation browser's stable slots.

## Status

implemented

## Slots

The list displays bundled documents in slots `0-44` and Close at `53`. A file
view renders content in `19-28`, Previous at `46` when available, Next at `48`
when available, Documentation at `49`, and Close at `53`.

## Boundary

There is no Main Menu root, Refresh, confirmation pair, daemon row, transfer,
or dynamic region. The explicit Close control is the only local menu action that
closes the inventory.
