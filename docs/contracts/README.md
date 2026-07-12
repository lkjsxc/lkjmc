# Cross-contract coverage

## Purpose

This area maps checked source-owned contracts to their owner documents.

## Table of contents

- [Command coverage](command-coverage.md)
- [Command registry](command-registry.md)
- [Command effect boundaries](command-effects.md)
- [Config schema coverage](config-schema.md)
- [Locale coverage](locale-coverage.md)
- [Menu documents](menu-documents.md)
- [Permission coverage](permission-coverage.md)

## Scope

Command contracts are the selected domain-sharded source: each bounded shard
names a command body member set, daemon registration, and supported consumer.
The source does not add an adapter or turn an absent binding into a command.
Rust owns product JSON configuration. The JVM mirror remains withdrawn.

## Proof levels

- **Contract:** checked source data or owner document is present.
- **Implementation:** an owner-source test proves a binding.
- **Deterministic:** a repository check runs without external services.
- **Compose** and **live:** environment-backed checks; a skip proves neither.

A lower proof level does not establish a higher one. Coverage records are in
`docs/execution/documentation-coverage/contracts.json`.

## Change rule

A command, consumer, config, locale, or menu change updates its owner contract
and deterministic check in the same change. Withdrawn Paper, Velocity, and
Discord daemon adapters remain explicit compatibility results, never bindings.
