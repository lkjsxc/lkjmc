# Data architecture

## Purpose

This area owns PostgreSQL usage, schema shape, migration rules, and durable data
contracts.

## Table of contents

- [PostgreSQL](postgres.md)
- [Schema](schema.md)

## Contract

PostgreSQL is the only product store. SQLite is not used for product state.
