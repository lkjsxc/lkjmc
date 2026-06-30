# `/lkjmc` completion

## Purpose

This document owns shared `/lkjmc` completion semantics for Paper/Folia tab
completion and Velocity Brigadier suggestions.

## Contract

Completion is stable, permission-filtered, case-insensitive, and generated from
the shared JVM command tree. Paper and Velocity may differ only where a command
is platform-specific, such as Velocity transfer commands.

## Inputs

The shared command tree receives immutable inputs:

- sender platform;
- normalized argument tokens, including a trailing empty token for a space;
- permission checker for the current sender;
- cached completion context containing server ids, online player names,
  templates, role ids, adventure ids, shop item ids, kit ids, vote ids, and safe
  principal hints.

## Edge cases

- `/lkjmc ` returns only permitted root literals.
- Mixed-case input matches lower-case literals and preserves canonical output.
- Exact literals followed by a space move to child suggestions.
- `server delete <server> confirm` suggests `confirm` only after a server token.
- Missing daemon context returns known literals and empty dynamic candidates.
- Hidden Brigadier nodes return product usage or no-permission copy, never parser
  position internals.

## Nonblocking rule

Paper completion refreshes dynamic context asynchronously and returns the last
known good cache while refresh work is in flight or failing. Daemon
authorization remains final at execution time.
