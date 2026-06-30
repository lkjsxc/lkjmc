# Orchestration architecture

## Purpose

This area owns desired state, presence, autosuspend, temporary instances,
reconciliation, and process runtime behavior.

## Table of contents

- [Desired state](desired-state.md)
- [Idle autosuspend](idle-autosuspend.md)
- [Kubernetes runtime](kubernetes-runtime.md)
- [Presence](presence.md)
- [Process runtime](process-runtime.md)
- [Temporary instances](temporary-instances.md)
- [Temporary runtime](temporary-runtime.md)

## Contract

Instance commands write durable intent in PostgreSQL. Pure Rust planners produce
effect descriptions. Daemon adapters execute process effects after state writes
and without holding runtime locks across PostgreSQL work.
