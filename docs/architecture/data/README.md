# Data architecture

## Purpose

This area owns PostgreSQL usage, schema shape, migration rules, and durable data
contracts.


## Status

implemented

## Table of contents

- [Claims](claims.md)
- [Economy](economy.md)
- [PostgreSQL](postgres.md)
- [Schema](schema.md)
- [Store](store.md)

## Current and target boundary

PostgreSQL is the only product store; SQLite is not product state. Store
helpers own persistence effects, while pure cores own validation and planning.
Plugins and transports request daemon work rather than embedding product SQL.

## Evidence and degraded behavior

Migrations and `lkjmc-store` are source evidence. If PostgreSQL is unavailable,
commands return the store failure and do not substitute plugin-local or
in-memory durable state.
