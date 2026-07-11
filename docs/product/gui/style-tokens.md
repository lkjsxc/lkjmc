# Style tokens

## Purpose

This document records the limited local documentation presentation boundary.

## Status

implemented

## Current boundary

The Paper documentation menu uses plugin-local item names and bundled document
titles. It does not provide a platform-neutral role system, player-locale
binding, daemon-derived text, Action Bar rendering, or a general inventory theme
palette.

## Local item roles

- `BOOK` selects a bundled document.
- `PAPER` renders a bundled document line.
- `ARROW` moves a document page.
- `BARRIER` is the explicit Close action.
- `NETHER_STAR` is the hard-locked local documentation token.

These roles do not authorize a command or mutation. Unknown local metadata is
inert.

## Verification

JVM containment and local menu checks prove the bounded presentation surface.
They do not prove a reusable menu styling framework.
