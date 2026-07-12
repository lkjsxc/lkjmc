# Command registry

## Purpose

The selected command source is a set of bounded JSON domain shards under
`contracts/commands/`. Each entry names one real daemon registration and its
closed, command-specific request schema. It does not prove a handler effect
completed.

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

`request.fields` is a command-local map. Every field declares `required` and
its JSON `type`; required fields must be present and every present value must
have that type. Wildcard `value` is forbidden: object-like members use a named,
closed domain shape that validates required nested members, nested types, and
unknown nested members. `request` is `handler-defined` only for commands whose
handler accepts no request members. This is not a family allowlist and does not
infer a field from a sibling command.

`lkjmc_core::command_registry` parses every listed shard. The closed validator
rejects a non-object body, undeclared member, missing required member, or wrong
type before a registered handler runs. Handler-specific semantic validation
continues in the handler, so an accepted member is not a success claim.

All registered-handler invocation paths enter the same closed dispatch path:
transport callers authenticate through `dispatch_as`; trusted in-process
workflows use `dispatch_internal`. Both perform registry lookup, authorization,
and request validation before choosing a registered handler. Callers do not
invoke a registered handler directly.

`scripts/check-contracts.py` compares the complete shard manifest with the
filesystem and generated Rust include list, then compares shards with daemon
registrations, CLI and web literals, compatibility results, menu catalog
documents, config ownership, and generated outputs. Truth probes parse every
literal CLI and web `json!` body and validate it against that command's closed
contract. `scripts/check-command-docs.py` is its command catalog compatibility
entrypoint.

## Boundary

The response body, effect completion, replay outcome, and timeout outcome remain
only as specific as the source-backed entry says. A value of `not-established`
is an explicit absence, not an implementation claim. Transport-authenticated
subjects and daemon authorization remain authoritative for identity decisions.

## Change procedure

Update the domain shard, actual handler, actual consumer when one exists, and
owner documentation together. Literal CLI and web bodies are checked against
their command schemas. Run the contract check and regenerate its checked
outputs. Do not add a planned command, a fake consumer, or a dynamic menu route.
