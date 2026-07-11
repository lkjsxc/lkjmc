# Architecture

## Purpose

This area owns system structure, component boundaries, data ownership, runtime
contracts, plugin contracts, assets, bootstrap, and security contracts.


## Status

implemented

## Table of contents

- [Overview](overview.md)
- [Assets](assets/README.md)
- [Bootstrap](bootstrap/README.md)
- [Data](data/README.md)
- [Orchestration](orchestration/README.md)
- [Plugin](plugin/README.md)
- [Runtime](runtime/README.md)
- [Security](security/README.md)
- [Views](views/README.md)
- [Web](web/README.md)

## Current boundary

PostgreSQL owns durable product state. Rust owns pure models and planners, the
store, daemon, CLI, and effect adapters. Java owns platform integration and
never becomes a second product store.

## Target boundary

Pure Rust decisions describe validation, desired state, and effects; adapters
perform database, filesystem, network, process, or cluster work only after
those decisions. Plugins and web/Discord surfaces request daemon commands.

## Evidence and degraded behavior

Cross-cutting [views](views/README.md) name exact implementation sources and
non-atomic boundaries. `check-docs.py` and `check-lines.py` check documentation
shape, not external effects. Live Minecraft, Discord, and Kubernetes proof is
opt-in; absent prerequisites must be reported as skipped, never as healthy or
complete.
