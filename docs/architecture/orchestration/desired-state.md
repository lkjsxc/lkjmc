# Desired state

## Purpose

This document defines target state planning for managed instances.

## Desired states

- `stopped`
- `starting`
- `running`
- `stopping`
- `restarting`
- `deleting`
- `failed`

## Observed states

- process absent
- process starting
- process healthy
- process unhealthy
- process exited
- process unknown

## Reconciliation

Target behavior: the daemon reads desired state, observed processes, and node
policy; pure Rust planning returns effects; adapters execute effects and write
observations.

Current behavior: instance commands update desired state and immediately execute
the local process effect for explicit launch profiles. A periodic reconciler is
not running yet.
