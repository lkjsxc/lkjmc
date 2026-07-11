# Design system

## Purpose

This contract defines the bounded visual language of the local documentation
menu.

## Status

implemented

## Local surfaces

The document list is a 54-slot inventory with bundled document books in the
first 45 slots and Close at `53`. A document page renders local paper lines,
Previous at `46` when available, Next at `48` when available, Documentation at
`49`, and Close at `53`. The token is a `NETHER_STAR` in hotbar slot `8`.

## Boundary

The local menu uses plugin-local display strings and metadata. It has no shared
menu renderer, theme registry, locale binding, dynamic region, confirmation
pair, refresh control, or daemon-derived lore. A future visual system must not
be treated as a shipped Java daemon surface without its own bounded contract.

## Verification

No repository catalog check proves bundled-resource consumption or control
rendering. Platform tests cover retained registration only; guarded playable
proof remains unavailable. These checks do not prove a general inventory design
system.
