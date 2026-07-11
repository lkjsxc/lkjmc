# Command registry

## Purpose

`contracts/commands.json` is the structural source of truth for daemon command
names. It defines a catalog contract; it does not itself prove handler behavior,
surface registration, or a successful external invocation.

## File format

The file contains one top-level object with a sorted `commands` array. Each
entry has:

- `name`: the exact daemon command literal.
- `family`: the product family used for grouping.
- `authorization`: `open`, `player`, `admin`, or `operator`.
- `surfaces`: any of `paper`, `velocity`, `cli`, `web`, `discord`.
- `doc`: repository-relative owner documentation path.
- `summary`: one-line behavior summary.
- `status`: the schema-defined strict, historical-compatibility, deprecated, or
  internal coverage state.
- `schemaCoverage`: the schema coverage tier.
- `requestSchema` and `responseSchema`: repository-relative JSON Schema files
  that define the accepted command body and response envelope for strict
  entries.

## Consumers

- `lkjmc-core::command_registry` parses and validates the file at test time and
  exposes typed read access for Rust callers.
- `lkjmc-daemon` dispatch registration tests are owner evidence for handler
  coverage; they are distinct from documentation checks.
- JVM common registry-resource tests are owner evidence for copied
  daemon-backed `CommandSpec` targets.
- `scripts/check-command-docs.py` deterministically checks ordering, fields,
  strict schema paths, owner-doc paths, and generated catalog parity. It does
  not invoke handlers or external surfaces.

## Proof boundary

A strict entry has request and response schema files and passes structural
checking. That is contract-level proof only. Implementation proof requires the
Rust dispatch and applicable JVM tests; Compose or live proof requires its
separately reported environment-backed command run. `authorization` classifies
an entry for catalog consumers; authenticated transport subject and daemon
authorization remain the authority for identity decisions.

## Change procedure

A command behavior change must update the contract, the real handler, and owner
documentation in the same change set. Do not add fake contract entries for
planned commands; target-only platform commands stay outside the registry until
a daemon handler exists.
