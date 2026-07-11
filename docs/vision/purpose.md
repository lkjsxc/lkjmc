# Purpose

## Purpose

This document states why `lkjmc` exists.

## Target intent

`lkjmc` should make a Minecraft network installable, observable, and operable
from a shell and from Minecraft itself. Its design center is one canonical
PostgreSQL-backed product store, one daemon-mediated control path, localized
player operations, and explicit effect boundaries.

## Principles

- Prefer truthful state, diagnostics, and recovery over apparent convenience.
- Put durable product truth and authorization behind daemon-owned contracts.
- Keep player interaction responsive by moving effects off scheduler callbacks.
- Make changes reviewable: an owner contract, bounded effect, and named proof.
- Treat external systems as conditional dependencies, never implied success.

## Current boundary

This is target direction. [State](../state/README.md) says what is implemented
now; owner areas supply the behavior and evidence before a target becomes state.
