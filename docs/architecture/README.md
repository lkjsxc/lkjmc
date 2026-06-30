# Architecture

## Purpose

This area owns system structure, component boundaries, data ownership, runtime
contracts, plugin contracts, assets, bootstrap, and security contracts.

## Table of contents

- [Overview](overview.md)
- [Assets](assets/README.md)
- [Bootstrap](bootstrap/README.md)
- [Data](data/README.md)
- [Orchestration](orchestration/README.md)
- [Plugin](plugin/README.md)
- [Runtime](runtime/README.md)
- [Security](security/README.md)
- [Web](web/README.md)

## Contract

PostgreSQL owns durable state. Rust owns core, store, daemon, CLI, and local
runtime. Java owns platform plugins and shared plugin-side contracts.
