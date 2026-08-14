# Minecraft commands

## Purpose

This document defines the currently shipped in-game command surface.

## Paper/Folia

- `/menu` opens the five-route local menu and requires `lkjmc.user.menu`.
- `/docs` opens the curated docs directory.
- `/docs search <query>` searches the bundled corpus.
- `/docs <path>` opens a bundled file or falls back to search.

The slot-8 token opens `root`. These entrypoints perform only local inventory
navigation and do not call the daemon.

## Velocity

Velocity owns `/lkjmc`:

- `/lkjmc` prints the two supported forms;
- `/lkjmc status` requests bounded network status and prints truthful unavailable
  or timeout feedback;
- `/lkjmc server hub` and `/lkjmc server survival` request a player transfer;
- completion offers only `status`, `server`, `hub`, and `survival` in the
  corresponding positions.

Unknown subcommands, extra arguments, non-player transfer sources, missing
routes, timeouts, and failed connections produce distinct feedback. A timeout
message does not mean the original transfer future completed.

No `/hub`, generic daemon command, admin command tree, economy, claim, home,
mail, party, or adventure command is part of this supported surface.

## Evidence boundary

Unit and platform tests prove parsing, completion, bounded continuations, and
registration behavior. Live logs prove installation and registration. Actual
command text, completion, status, and transfer still require an authorized
online-mode player and are not yet player-accepted.
