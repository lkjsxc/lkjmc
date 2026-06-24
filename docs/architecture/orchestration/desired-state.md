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

The daemon reads desired state, observed processes, and node policy; pure Rust
planning returns effects; adapters execute effects and write observations.
