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

## Contract

PostgreSQL is the only product store. SQLite is not used for product state.
Feature code should use `lkjmc-store` helpers rather than embedding product SQL
in plugins.
