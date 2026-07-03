# Cross-contract coverage

## Purpose

This area maps source-owned registries to documentation contracts.

## Table of contents

- [Command coverage](command-coverage.md)
- [Command registry](command-registry.md)
- [Config schema coverage](config-schema.md)
- [Locale coverage](locale-coverage.md)
- [Permission coverage](permission-coverage.md)

## Contract

Command, permission, config, and locale docs must be checked by deterministic
scripts in default verification. Daemon command names are sourced from
`contracts/commands.json`; code and docs checks consume that registry rather
than scraping implementation files.
