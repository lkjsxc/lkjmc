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
- `surfaces`: any of `paper`, `velocity`, `cli`, or `web`. Discord command
  delegation is withdrawn until a trusted interaction policy exists.
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

## Schema coverage boundary

All 139 current `strict` entries declare `schemaCoverage: "generic-v&#49;"` and
share `contracts/schemas/command-request.schema.json` and
`contracts/schemas/command-response.schema.json`. This is **partial** schema
coverage: the request schema accepts any JSON object and the response schema
only fixes the envelope. Handler-specific required fields, value ranges,
unknown-field handling, effect class, idempotency, and deadline semantics are
not expressed by that generic schema tier.

A strict entry therefore has contract-level registry and envelope proof only.
Implementation proof requires the Rust dispatch and applicable JVM tests;
Compose or live proof requires its separately reported environment-backed
command run. `authorization` classifies an entry for catalog consumers;
authenticated transport subject and daemon authorization remain the authority
for identity decisions. Effect and delivery boundaries are in
[command-effects.md](command-effects.md).

## Reopened schema boundary

The generic tier is a rejected shape, not a claim that strict commands have
useful bodies. Before a strict command can claim typed coverage, its request and
response fields, validation limits, effects, retries, and deadline semantics
must be checked against the real handler. Every applicable adapter must have a
generated or checked consumer, and a consumer may not accept a body that its
handler rejects. `F-CLAIM-PROBES` starts with negative evidence for this gap; it
does not make the current registry product proof or relax the change procedure.

The normal `check-truth-probes.py --probe generic-schema-rejected` and
`--probe payload-consumers-required` commands reject the current generic tier
and absent consumer inventory. They become adoption gates only after the owner
work supplies typed schemas and checked consumers.

## Change procedure

A command behavior change must update the contract, the real handler, and owner
documentation in the same change set. Do not add fake contract entries for
planned commands; target-only platform commands stay outside the registry until
a daemon handler exists.
