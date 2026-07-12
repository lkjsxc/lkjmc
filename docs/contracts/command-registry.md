# Command registry

## Purpose

The selected command source is a set of bounded JSON domain shards under
`contracts/commands/`. Each entry names one real daemon registration and its
closed request-member set. It does not prove a handler effect completed.

## Current source contract

The checked registry has 137 daemon registrations at this revision. Shards are
listed by `contracts/commands/README.json`; no monolithic registry or generic
payload schema is a source of truth. Each command records its handler literal,
authorization class, supported consumer surfaces, request members, response
boundary, identity boundary, effect boundary, idempotency result, and deadline
result.

`internal` means no checked public consumer currently invokes that command.
`cli` and `web` appear only when their real Rust consumer contains the literal.
Paper, Velocity, and Discord daemon consumers are withdrawn compatibility
results, not generated bindings.

## Validation

`lkjmc_core::command_registry` parses every listed shard. Its closed-object
validator rejects a non-object body, an undeclared member, or a missing required
member before a registered handler runs. Handler-specific semantic validation
continues in the handler, so an accepted member is not a success claim.

`scripts/check-contracts.py` compares shards with daemon registrations, CLI and
web literals, compatibility results, menu catalog documents, config ownership,
and generated outputs. `scripts/check-command-docs.py` is its command catalog
compatibility entrypoint.

## Boundary

The response body, effect completion, replay outcome, and timeout outcome remain
only as specific as the source-backed entry says. A value of `not-established`
is an explicit absence, not an implementation claim. Transport-authenticated
subjects and daemon authorization remain authoritative for identity decisions.

## Change procedure

Update the domain shard, actual handler, actual consumer when one exists, and
owner documentation together. Run the contract check and regenerate its checked
outputs. Do not add a planned command, a fake consumer, or a dynamic menu route.
