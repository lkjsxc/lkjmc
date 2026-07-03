# Command registry

## Purpose

`contracts/commands.json` is the structural source of truth for daemon command
names. It prevents a handler, Java command target, or daemon catalog entry from
shipping without the same public contract.

## File format

The file contains one top-level object with a sorted `commands` array. Each
entry has:

- `name`: the exact daemon command literal.
- `family`: the product family used for grouping.
- `authorization`: `open`, `player`, `admin`, or `operator`.
- `surfaces`: any of `paper`, `velocity`, `cli`, `web`, `discord`.
- `doc`: repository-relative owner documentation path.
- `summary`: one-line behavior summary.

## Consumers

- `lkjmc-core::command_registry` parses and validates the file at test time and
  exposes typed read access for Rust callers.
- `lkjmc-daemon` dispatch registration tests assert every handler is present in
  the contract and every contract entry has a handler.
- JVM common tests compare daemon-backed `CommandSpec` targets with the copied
  registry resource.
- `scripts/check-command-docs.py` checks the daemon command catalog and `doc`
  paths from the registry instead of scraping source code.

## Change procedure

A command behavior change must update the contract, the real handler, and owner
documentation in the same change set. Do not add fake contract entries for
planned commands; target-only platform commands stay outside the registry until
a daemon handler exists.
