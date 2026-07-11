# Cross-contract coverage

## Purpose

This area maps source-owned registries to documentation contracts.

## Table of contents

- [Command coverage](command-coverage.md)
- [Command registry](command-registry.md)
- [Config schema coverage](config-schema.md)
- [Locale coverage](locale-coverage.md)
- [Menu documents](menu-documents.md)
- [Permission coverage](permission-coverage.md)

## Scope

These are structural contracts, not a claim that every listed capability has
been exercised. The daemon command name registry is `contracts/commands.json`;
Rust and JVM registration tests, rather than documentation parsing, establish
implementation bindings. Permission nodes describe adapter-visible grants, while
transport-authenticated subjects establish daemon identity. Rust owns product
JSON configuration; the JVM mirror covers only fields consumed by plugins.

## Proof levels

- **Contract:** a checked registry, schema, or owner document is present.
- **Implementation:** an owner source test proves a binding to that contract.
- **Deterministic:** a repository check runs without external services.
- **Compose** and **live:** environment-backed checks; a guarded skip is not
  proof at either level.

Do not infer a higher level from a lower one. Owner evidence and exact checks
are recorded in `docs/execution/documentation-coverage/contracts.json`.

## Change rule

Command, menu, permission, config, and locale changes must update their owner
contract and deterministic check evidence. A planned command, identity source,
or config field must not be represented as implemented before its owner source
and proof exist.
