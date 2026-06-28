# Command and menu runtime

## Purpose

This task owns the user-reported in-game command and inventory menu defects.

## Defects

- `/lkjmc status` and `/lkjmc server ...` can surface parser-position errors.
- Completion does not reliably show the intended `/lkjmc` tree.
- Paper can expose adapter-flavored command namespace text.
- Dynamic menus collapse dependency failures into one daemon-unavailable row.
- Some menu actions close inventories without an explicit close click.
- The slot `8` hotbar token uses a compass instead of a nether star.

## Contract to implement

- `/lkjmc` is the only documented public control root on Paper/Folia and
  Velocity; adapter names remain build internals.
- A shared JVM command tree owns path, permission, sender kind, usage,
  execution target, and completion metadata.
- Paper/Folia and Velocity consume that tree for execution and suggestions.
- Valid documented syntax returns product messages, never parser internals.
- Dynamic menus render real data, true empty states, permission states, or typed
  diagnostics for daemon, auth, database, command, schema, and HTTP failures.
- No ordinary inventory action closes the menu; only `MenuAction.Close` or a
  manual player close may do so.
- The hotbar token in slot `8` is a nether star with persistent metadata.

## Acceptance gates

- Unit tests cover parser success, parser usage failures, and completions for
  `/lkjmc status`, `/lkjmc server list`, lifecycle commands, `confirm`, and
  proxy transfer commands.
- Menu tests cover typed unavailable rows, empty states, and close-effect
  isolation.
- Adapter code registers tab completion or suggestions from the shared tree.
- Token tests assert `NETHER_STAR` plus the persistent marker.
- `./scripts/check-lines.py`, `./scripts/check-docs.py`, and the relevant JVM
  tests pass before the blocker is closed.
