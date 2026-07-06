# Style tokens

## Purpose

This document defines platform-neutral visual roles, palette defaults, and text
rendering conventions for the implemented menu engine.

## Status

implemented

## Text pipeline

`TextRef.Key` resolves through the player's locale catalog with argument
substitution, then parses MiniMessage into an Adventure Component. `TextRef`
values that come from player data may appear only inside keyed templates.
Bindings must not emit complete English sentences as literals.

A shared MiniMessage helper owns parsing for inventory titles, item names, lore,
action bar frames, chat feedback, and proxy messages where a catalog key exists.
Item names are rendered as not italic by default so locale files do not carry
format reset noise.

## Roles

Roles style items; they never decide behavior.

| Role | Default styling | Use |
|---|---|---|
| `info` | gold | state and summaries |
| `navigation` | aqua | route changes, Back, Main Menu |
| `action` | green | enabled operations |
| `success` | bold green | confirm or completed action |
| `danger` | red | destructive intent |
| `disabled` | dark gray | unavailable action with reason |
| `decoration` | blank | inert border panes |

## Theme palette

Theme names map to stained-glass pane colors: `root` light blue, `network` cyan,
`travel` green, `claims` lime, `economy` yellow, `social` purple, `profile`
orange, `settings` light gray, `staff` red, `adventure` magenta, `danger` red,
and `docs` brown.

## Lore conventions

The first lore line states purpose in gray. Data lines use gray labels with
white values. The final line is an action hint, yellow when enabled or dark gray
when disabled. Disabled lore must include the exact reason and next possible
step.

## Forbidden rendering

Section-sign color codes, `ChatColor`, `setDisplayName(String)`, and
`setLore(List<String>)` are not allowed for engine items. The catalog is the
only source of player-visible sentences; the key itself is the last-resort
fallback.

## Verification

Catalog tests parse every English and Japanese value with strict MiniMessage.
Binding tests assert emitted keys exist and literal-only lore lines do not hide
English labels.
